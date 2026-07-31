package platform

import dataClasses.PlaybackSnapshot
import dataClasses.TrackInfo
import enums.PlaybackRepeat
import enums.PlaybackState
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import platform.interfaces.AudioPlayer
import sealedInterfaces.PlaybackEvent
import sealedInterfaces.PlaybackFailure
import uniffi.player_core.BandsListener
import uniffi.player_core.NetworkException
import uniffi.player_core.equalizerBands as readEqualizerBands
import uniffi.player_core.equalizerMaxDb as readEqualizerMaxDb
import uniffi.player_core.outputDevices as listOutputDevices
import uniffi.player_core.trackCover as readTrackCover
import uniffi.player_core.Player
import uniffi.player_core.PlayerEvent
import uniffi.player_core.PlayerException
import uniffi.player_core.PlayerListener
import uniffi.player_core.PlayerSnapshot
import uniffi.player_core.PlayerState
import uniffi.player_core.QuicSession
import uniffi.player_core.RepeatMode
import uniffi.player_core.TrackData

/**
 * Прослойка над плеером из Rust: переводит типы в обе стороны и владеет временем жизни
 * сгенерированного объекта. Дальше этого файла сгенерированные типы не уходят.
 *
 * Создание открывает устройство вывода и бросает [PlaybackFailure.Device], если открыть
 * его не вышло. Сервер к этому моменту не нужен: подключение задаётся позже, [connect]. На Android до этого момента должен быть вызван
 * `music.player.NativeContext.install`, иначе cpal не доберётся до параметров устройства.
 */
class AudioPlayerImpl : AudioPlayer {

    /**
     * Ящик событий. Он неограничен: обратный вызов приходит из управляющего потока Rust,
     * и задерживать его нельзя — пока получатель думает, команды не разбираются.
     */
    private val updates = Channel<PlaybackEvent>(Channel.UNLIMITED)

    private val spectrum = MutableStateFlow<List<Double>>(emptyList())

    /**
     * Rust-плеер создаётся не сразу, а при первом обращении, и это важно: его создание открывает
     * устройство вывода и ждёт, пока оно откроется. Поле, которое считается при сборке объекта,
     * считалось бы в фабрике графа зависимостей — то есть в композиции, на главном потоке, — и
     * держало бы его всё это время. А первым к плееру обращается один из сборщиков репозитория, и
     * все они на `Dispatchers.IO`.
     *
     * Попутно меняется судьба отказа устройства: раньше он вылетал из фабрики и ронял приложение,
     * а теперь попадает в обычный перехват вокруг приказа и доходит до экрана надписью.
     */
    private val opened = lazy {
        guard {
            Player(
                object : PlayerListener {
                    override fun onEvent(event: PlayerEvent) {
                        // Отказ бывает только у закрытого ящика, то есть после `close`.
                        updates.trySend(playbackEvent(event))
                    }
                },
                object : BandsListener {
                    override fun onBands(bands: List<Double>) {
                        spectrum.value = bands
                    }
                },
                // Подключение к серверу задаётся отдельно, через `connect`: адрес вводят
                // позже плеера и меняют на ходу.
                null,
            )
        }
    }

    private val player: Player get() = opened.value

    /**
     * Своё подключение к серверу. Оно у плеера собственное, а не общее с сетевым
     * репозиторием: тот держит своё для плейлиста и уведомлений сервера, а звук идёт
     * отдельным. Цена — два подключения к одному серверу вместо одного.
     *
     * Поле меняется только в [connect], и звать его положено из одной корутины: два
     * подключения подряд не должны перегонять друг друга.
     */
    private var session: QuicSession? = null

    override val events: Flow<PlaybackEvent> = updates.receiveAsFlow()

    override val bands: StateFlow<List<Double>> = spectrum

    override fun setQueue(tracks: List<TrackInfo>, start: Int?) = guard {
        player.setQueue(tracks.map { track -> trackData(track) }, start?.let { trackNumber(it) })
    }

    override fun play() = guard { player.play() }

    override fun pause() = guard { player.pause() }

    override fun playTrack(index: Int) = guard { println("playerKotlin track №$index"); player.playTrack(trackNumber(index)) }

    override fun seek(positionMs: Long) = guard {
        if (positionMs < 0) {
            throw PlaybackFailure.BadArgument("время не бывает отрицательным: $positionMs")
        }
        player.seek(positionMs.toULong())
    }

    override fun next() = guard { player.next() }

    override fun previous() = guard { player.previous() }

    override fun stop() = guard { player.stop() }

    override fun setShuffle(shuffle: Boolean) = guard { player.setShuffle(shuffle) }

    override fun setRepeat(repeat: PlaybackRepeat) = guard {
        player.setRepeat(repeatMode(repeat))
    }

    override fun connect(address: String) {
        val opened = if (address.isBlank()) null else openSession(address)
        // Сначала новое, потом закрыть прежнее: пока по нему дочитывается дорожка, его
        // держит сам Rust, и закрытие отсюда только отпускает нашу ручку.
        guard { player.setSession(opened) }
        session?.close()
        session = opened
    }

    // Сгенерированная функция называется так же, как этот метод, поэтому импортирована под
    // псевдонимом: без него `outputDevices()` в теле разрешилось бы в сам этот метод.
    override fun outputDevices(): List<String> = guard { listOutputDevices() }

    override fun setOutputDevice(name: String?) = guard { player.setOutputDevice(name) }

    override fun setVolume(volume: Float) = guard { player.setVolume(volume) }

    // Как и со списком устройств: сгенерированные функции называются так же, как методы, поэтому
    // импортированы под псевдонимами. Отказать они не могут, и ловить здесь нечего.
    override fun equalizerBands(): List<Float> = readEqualizerBands()

    override fun equalizerMaxDb(): Float = readEqualizerMaxDb()

    override fun setEqualizer(gainsDb: List<Float>) = guard { player.setEqualizer(gainsDb) }

    // Как и со списком устройств: сгенерированная функция называется так же, как метод, поэтому
    // импортирована под псевдонимом.
    override fun trackCover(path: String): ByteArray? = guard { readTrackCover(path) }

    override fun snapshot(): PlaybackSnapshot = guard { playbackSnapshot(player.snapshot()) }

    /**
     * Отпускает объект Rust. Тот велит своим потокам выключиться и дожидается их, поэтому
     * вызов возвращается не мгновенно.
     */
    override fun close() {
        updates.close()
        // Плеера может и не быть: до первого обращения его не создают, а закрыть просят всегда.
        if (opened.isInitialized()) {
            player.close()
        }
        session?.close()
        session = null
    }

    /**
     * Подключается к серверу, переводя беду в свою: адрес набирает человек, и «адрес не
     * разобран» — это довод вызова, а «сервер не ответил» — беда подключения.
     */
    private fun openSession(address: String): QuicSession =
        try {
            QuicSession.connect(address)
        } catch (failure: NetworkException) {
            throw when (failure) {
                is NetworkException.Address -> PlaybackFailure.BadArgument(failure.reason)
                is NetworkException.Connect -> PlaybackFailure.Server(failure.reason)
            }
        }

    /**
     * Ловит сгенерированное исключение и бросает своё: дальше этого класса
     * [PlayerException] не идёт.
     */
    private fun <T> guard(body: () -> T): T =
        try {
            body()
        } catch (failure: PlayerException) {
            throw playbackFailure(failure)
        }

    private fun playbackFailure(failure: PlayerException): PlaybackFailure = when (failure) {
        is PlayerException.Device -> PlaybackFailure.Device(failure.reason)
        is PlayerException.Track -> PlaybackFailure.Track(failure.reason)
        is PlayerException.BadArgument -> PlaybackFailure.BadArgument(failure.reason)
        is PlayerException.Gone -> PlaybackFailure.Gone(failure.reason)
    }

    private fun playbackEvent(event: PlayerEvent): PlaybackEvent = when (event) {
        is PlayerEvent.StateChanged -> PlaybackEvent.StateChanged(playbackState(event.state))
        is PlayerEvent.TrackStarted -> PlaybackEvent.TrackStarted(event.index.toInt())
        is PlayerEvent.TrackFinished -> PlaybackEvent.TrackFinished
        is PlayerEvent.QueueFinished -> PlaybackEvent.QueueFinished
        is PlayerEvent.Failed -> PlaybackEvent.Failed(playbackFailure(event.error))
        is PlayerEvent.PositionChanged -> PlaybackEvent.PositionChanged
    }

    private fun playbackSnapshot(snapshot: PlayerSnapshot) = PlaybackSnapshot(
        state = playbackState(snapshot.state),
        positionMs = snapshot.positionMs.toLong(),
        durationMs = snapshot.durationMs?.toLong(),
        trackIndex = snapshot.trackIndex?.toInt(),
        trackCount = snapshot.trackCount.toInt(),
        hasNext = snapshot.hasNext,
        hasPrevious = snapshot.hasPrevious,
        shuffle = snapshot.shuffle,
        repeat = playbackRepeat(snapshot.repeat),
    )

    private fun playbackState(state: PlayerState) = when (state) {
        PlayerState.IDLE -> PlaybackState.IDLE
        PlayerState.PLAYING -> PlaybackState.PLAYING
        PlayerState.PAUSED -> PlaybackState.PAUSED
    }

    private fun playbackRepeat(repeat: RepeatMode) = when (repeat) {
        RepeatMode.OFF -> PlaybackRepeat.OFF
        RepeatMode.ONE -> PlaybackRepeat.ONE
        RepeatMode.ALL -> PlaybackRepeat.ALL
    }

    private fun repeatMode(repeat: PlaybackRepeat) = when (repeat) {
        PlaybackRepeat.OFF -> RepeatMode.OFF
        PlaybackRepeat.ONE -> RepeatMode.ONE
        PlaybackRepeat.ALL -> RepeatMode.ALL
    }

    private fun trackData(track: TrackInfo) = TrackData(
        trackUrl = track.trackUrl,
        trackName = track.trackName,
        trackIndex = track.trackIndex,
        isLocal = track.isLocal,
    )

    /**
     * Номер дорожки через границу идёт беззнаковым, и отрицательное число превратилось бы
     * в миллиард с лишним: Rust ответил бы «такой дорожки нет» вместо правды о доводе.
     */
    private fun trackNumber(index: Int): UInt {
        if (index < 0) {
            throw PlaybackFailure.BadArgument("номер дорожки не бывает отрицательным: $index")
        }
        return index.toUInt()
    }
}
