package repository

import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.decodeToImageBitmap
import constants.PlayerConstantsLongs
import dataClasses.PlaybackSnapshot
import dataClasses.TrackInfo
import enums.PlaybackRepeat
import enums.PlaybackState
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import platform.interfaces.AudioPlayer
import platform.interfaces.BackgroundPlayback
import repository.interfaces.NetworkRepository
import repository.interfaces.OfflineRepository
import repository.interfaces.SettingsRepository
import repository.interfaces.SongsRepository
import sealedInterfaces.PlaybackEvent
import sealedInterfaces.PlaybackFailure

/**
 * Воспроизведение поверх плеера из Rust.
 *
 * Очередь, перемешивание и повтор живут в Rust: здесь их не считают, а только приказывают
 * и показывают. Состояние берётся из снимка — событием, когда плеер о чём-то сообщил, и
 * опросом, пока дорожка играет: позиция считается в момент вызова и сама не приходит.
 */
class SongsRepositoryImplRust(
    private val scope: CoroutineScope,
    private val player: AudioPlayer,
    private val background: BackgroundPlayback,
    private val networkRepository: NetworkRepository,
    private val settings: SettingsRepository,
    private val offlineRepository: OfflineRepository,
) : SongsRepository {

    // Начальное состояние берётся не у плеера, а отсюда, и это не мелочь: спросить снимок значило
    // бы создать плеер здесь же, в конструкторе, то есть на главном потоке — а его создание
    // открывает устройство вывода и ждёт. Пока плеера нет, правда именно такая: ничего не играет.
    private val playback = MutableStateFlow(
        PlaybackSnapshot(
            state = PlaybackState.IDLE,
            positionMs = 0,
            durationMs = null,
            trackIndex = null,
            trackCount = 0,
            hasNext = false,
            hasPrevious = false,
            shuffle = false,
            repeat = PlaybackRepeat.OFF,
        )
    )
    private val playerComplaint = MutableStateFlow("")

    /**
     * Ждёт ли автоплей списка. Берётся из настройки при запуске — включение посреди работы
     * подействует со следующего запуска, — и снимается, когда дождался или когда пользователь
     * нажал что-нибудь в интерфейсе раньше ([stopAutoplay]).
     */
    private val autoplayPending = MutableStateFlow(settings.autoplay.value)
    private val onlineSongs = MutableStateFlow<List<TrackInfo>>(emptyList())
    private val outputs = MutableStateFlow<List<String>>(emptyList())
    private val coverBytes = MutableStateFlow<ByteArray?>(null)

    override val isLocal: StateFlow<Boolean> = settings.isPlayingLocal
    override val songListOffline: StateFlow<List<TrackInfo>> = offlineRepository.offlineSongsFlow
    override val songListOnline: StateFlow<List<TrackInfo>> = onlineSongs
    override val playbackError: StateFlow<String> = playerComplaint
    override val outputDevices: StateFlow<List<String>> = outputs
    override val cover: StateFlow<ByteArray?> = coverBytes

    // Постоянные ядра: спрашиваются один раз, плеер для этого не создаётся.
    override val equalizerBands: List<Float> = player.equalizerBands()
    override val equalizerMaxDb: Float = player.equalizerMaxDb()

    override val equalizerGains: StateFlow<List<Float>> = settings.equalizer
        .map(::fitEqualizer)
        .stateIn(scope, SharingStarted.Eagerly, fitEqualizer(settings.equalizer.value))

    /**
     * Разбор не на главном потоке: обложка на несколько тысяч точек распаковывается долго, и
     * всё это время экран стоял бы. Раньше разбор шёл в композиции, то есть там.
     */
    override val coverImage: StateFlow<ImageBitmap?> = coverBytes
        .map { bytes -> bytes?.let(::decodeCover) }
        .flowOn(Dispatchers.Default)
        .stateIn(scope, SharingStarted.WhileSubscribed(), null)

    override val playlist: StateFlow<List<TrackInfo>> =
        combine(isLocal, songListOffline, songListOnline) { local, offline, online ->
            if (local) offline else online
        }.stateIn(scope, SharingStarted.Eagerly, emptyList())

    override val isPlaying: StateFlow<Boolean> = playback
        .map { snapshot -> snapshot.state == PlaybackState.PLAYING }
        .stateIn(scope, SharingStarted.Eagerly, false)

    override val currentPositionMs: StateFlow<Long> = playback
        .map { snapshot -> snapshot.positionMs }
        .stateIn(scope, SharingStarted.Eagerly, 0L)

    /** Ноль означает «длительность неизвестна»: контейнер её не объявил. */
    override val currentTrackLength: StateFlow<Long> = playback
        .map { snapshot -> snapshot.durationMs ?: 0L }
        .stateIn(scope, SharingStarted.Eagerly, 0L)

    override val shuffle: StateFlow<Boolean> = playback
        .map { snapshot -> snapshot.shuffle }
        .stateIn(scope, SharingStarted.Eagerly, false)

    /** Экран знает только «повторять или нет», и это повтор одной дорожки. */
    override val repeat: StateFlow<Boolean> = playback
        .map { snapshot -> snapshot.repeat == PlaybackRepeat.ONE }
        .stateIn(scope, SharingStarted.Eagerly, false)

    override val currentPlay: StateFlow<TrackInfo?> =
        combine(playback, playlist) { snapshot, tracks ->
            snapshot.trackIndex?.let { index -> tracks.getOrNull(index) }
        }.stateIn(scope, SharingStarted.Eagerly, null)

    /**
     * Полосы для визуализатора. Их присылает плеер сам, с темпом звука; здесь остаётся
     * только погасить их, когда играть нечего, — на паузе картинка замирает, а в тишине
     * её быть не должно.
     */
    override val equalizerArray: StateFlow<List<Double>> =
        combine(player.bands, playback) { bands, snapshot ->
            if (snapshot.state == PlaybackState.IDLE) emptyList() else bands
        }.stateIn(scope, SharingStarted.WhileSubscribed(), emptyList())

    init {
        // Очередь в Rust — это список, из которого играют. Сменился список или сторона
        // (свои файлы или сервер) — очередь задаётся заново. Приказ исполняется в сборщике, а не
        // через command: два списка подряд не должны перегнать друг друга, иначе прежний лёг бы
        // поверх нового — а при автоплее пустой список при запуске оборвал бы начатую дорожку.
        //
        // Автоплей ждёт здесь же, а не отдельным сборщиком: отдельный мог бы приказать «играй
        // дорожку N» раньше, чем эта очередь дошла до плеера. Пока он ждёт, каждый список
        // проверяется: пустой — ждать дальше, непустой — начать со случайной дорожки и больше не
        // ждать. При запуске список пуст: папку сканирует наблюдатель, а с сервера список
        // приходит после подключения.
        scope.launch(Dispatchers.IO) {
            playlist.collect { tracks ->
                val start = if (autoplayPending.value) tracks.indices.randomOrNull() else null
                if (start != null) {
                    autoplayPending.value = false
                }
                execute {
                    player.setQueue(tracks, start)
                    if (start != null) {
                        player.play()
                    }
                }
            }
        }

        // Перемешивание и повтор записаны в настройках и переживают перезапуск. Плееру они
        // отдаются отсюда — при запуске и при каждой смене, — а кнопки показывают снимок плеера:
        // нажатая кнопка значит, что режим включён в самом плеере, а не только записан. Режимы
        // переживают и смену очереди: Rust перемешивает новый список сам, если перемешивание
        // включено.
        scope.launch(Dispatchers.IO) {
            settings.shuffle.collect { on -> execute { player.setShuffle(on) } }
        }
        scope.launch(Dispatchers.IO) {
            settings.repeat.collect { on ->
                execute { player.setRepeat(if (on) PlaybackRepeat.ONE else PlaybackRepeat.OFF) }
            }
        }

        // Звук пошёл — службу надо поднять: без неё система вольна прибить процесс в
        // фоне вместе с воспроизведением. Поток состояния не повторяет одинаковое,
        // поэтому вызов приходится ровно на переход к игре.
        scope.launch(Dispatchers.Main) {
            isPlaying.collect { playing -> if (playing) background.start() }
        }

        scope.launch(Dispatchers.IO) {
            player.events.collect { event ->
                if (event is PlaybackEvent.Failed) {
                    playerComplaint.value = event.failure.reason
                }
                refresh()
            }
        }

        // Опрос позиции. Пока дорожка не играет, он не идёт: позиция стоит, и спрашивать
        // о ней нечего.
        scope.launch(Dispatchers.IO) {
            playback
                .map { snapshot -> snapshot.state == PlaybackState.PLAYING }
                .distinctUntilChanged()
                .collectLatest { playing ->
                    while (playing) {
                        delay(PlayerConstantsLongs.PositionPollMs.value)
                        refresh()
                    }
                }
        }

        // Устройство вывода: выбранное настройкой отдаётся плееру. Пустая строка означает
        // основное устройство системы. Приказ исполняется здесь же, а не через command: сбор идёт
        // одной корутиной, и два выбора подряд не перегонят друг друга.
        scope.launch(Dispatchers.IO) {
            settings.outputDevice.collect { name ->
                execute { player.setOutputDevice(name.ifBlank { null }) }
            }
        }

        // Обложка играющей дорожки. Читается при каждой смене: держать их все незачем — картинки
        // бывают по несколько мегабайт, а показывается одна.
        scope.launch(Dispatchers.Main.immediate) {
            currentPlay.collect { track ->
                if (track == null || !track.isLocal) {
                    // У дорожки с сервера обложки нет: в протоколе такого запроса не бывает.
                    coverBytes.value = null
                } else {
                    command { coverBytes.value = player.trackCover(track.trackUrl) }
                }
            }
        }

        // Громкость приложения. Отдаётся плеером сразу, очереди приказов не ждёт. Исполняется в
        // сборщике по той же причине, что и устройство: иначе прежнее значение могло бы лечь
        // поверх нового.
        scope.launch(Dispatchers.IO) {
            settings.volume.collect { volume ->
                execute { player.setVolume(volume) }
            }
        }

        // Эквалайзер — как громкость: плеер применяет сразу, а сборщик один, и прежний набор не
        // ляжет поверх нового.
        scope.launch(Dispatchers.IO) {
            equalizerGains.collect { gains ->
                execute { player.setEqualizer(gains) }
            }
        }

        // Адрес сервера вводят уже после запуска и меняют на ходу. Сбор идёт одной
        // корутиной, а не через command: два подключения подряд не должны перегонять
        // друг друга.
        scope.launch(Dispatchers.IO) {
            settings.serverAddress.collect { address ->
                try {
                    player.connect(address)
                } catch (failure: PlaybackFailure) {
                    playerComplaint.value = failure.reason
                }
            }
        }

        scope.launch(Dispatchers.IO) {
            networkRepository.observeServerUpdates().collect {
                networkRepository.getServerConfig()
                onlineSongs.value = networkRepository.fetchPlaylist()
            }
        }
    }

    /**
     * Нажатие на дорожку: на ту же — пауза и продолжение, на другую — переход к ней.
     */
    override fun play(trackInfo: TrackInfo) {
        if (currentPlay.value != trackInfo) {
            command { player.playTrack(trackInfo.trackIndex) }
        } else if (isPlaying.value) {
            command { player.pause() }
        } else {
            command { player.play() }
        }
    }

    override fun setPlaying(playing: Boolean) = command {
        if (playing) player.play() else player.pause()
    }

    /**
     * Снимок берётся здесь же, в том же приказе: между вопросом «играет ли» и ответом на
     * него ничего не успевает случиться.
     */
    override fun togglePlaying() = command {
        if (player.snapshot().state == PlaybackState.PLAYING) player.pause() else player.play()
    }

    /**
     * Записанные усиления, приведённые к числу полос плеера: недописанные полосы — на нуле. Лишние
     * отбрасываются — они остались бы от ядра, у которого полос было больше.
     */
    private fun fitEqualizer(stored: List<Float>): List<Float> =
        List(equalizerBands.size) { band -> stored.getOrElse(band) { 0f } }

    /**
     * Перечисление устройств — разговор с системой у каждого из них, поэтому оно уходит в
     * `Dispatchers.IO` и делается только по просьбе, а не само.
     */
    override fun refreshOutputDevices() = command {
        outputs.value = player.outputDevices()
    }

    /**
     * Кнопка на экране при перемешивании ещё сообщает о ручном выборе (`onTrackSelectedManually`),
     * но здесь это пустой вызов: порядком распоряжается Rust. Поэтому его нет и тут.
     */
    override fun playOrPause() {
        val current = currentPlay.value
        if (current != null) {
            play(current)
        } else {
            playlist.value.randomOrNull()?.let(::play)
        }
    }

    override fun nextTrack() = command { player.next() }

    override fun prevTrack() = command { player.previous() }

    /** Переключается настройка, а плеер получает её в сборщике из `init`. */
    override fun shuffle() {
        scope.launch(Dispatchers.IO) { settings.toggleShuffle() }
    }

    override fun repeat() {
        scope.launch(Dispatchers.IO) { settings.toggleRepeat() }
    }

    override fun seekBy(deltaMs: Long) {
        val length = currentTrackLength.value
        if (length <= 0L) return
        val target = (currentPositionMs.value + deltaMs).coerceIn(0L, length)
        command { player.seek(target) }
    }

    /** Ползунок отдаёт долю дорожки, плеер принимает миллисекунды. */
    override fun seekTo(percentage: Float) {
        val length = currentTrackLength.value
        if (length <= 0L) return
        command { player.seek((length * percentage).toLong()) }
    }

    /**
     * Порядком воспроизведения распоряжается Rust: при перемешивании он сам ставит
     * выбранную дорожку первой. Считать здесь нечего.
     */
    override fun onTrackSelectedManually(index: Int) = Unit

    override fun stopAutoplay() {
        autoplayPending.value = false
    }

    /**
     * Приказ плееру. Он уходит в свой поток, возврата не ждёт и может не получиться —
     * поэтому беда ловится здесь и идёт на экран, а не в никуда.
     */
    private fun command(body: () -> Unit) {
        scope.launch(Dispatchers.IO) { execute(body) }
    }

    /**
     * То же, что [command], но на месте: для сборщиков, у которых порядок приказов важен. Два
     * запуска `command` подряд — две корутины, и исполниться они могут в любом порядке.
     */
    private fun execute(body: () -> Unit) {
        try {
            body()
            playback.value = player.snapshot()
        } catch (failure: PlaybackFailure) {
            playerComplaint.value = failure.reason
        }
    }

    /**
     * Неудача разбора здесь не беда, а обычное дело: в файле лежит что угодно, и картинкой это
     * быть не обязано — встречаются и форматы, которых система не знает. Показать вместо
     * обложки надо всего лишь ничего: под ней нарисованная пластинка.
     */
    private fun decodeCover(bytes: ByteArray): ImageBitmap? =
        try {
            bytes.decodeToImageBitmap()
        } catch (failure: IllegalArgumentException) {
            null
        }

    private fun refresh() {
        try {
            playback.value = player.snapshot()
        } catch (failure: PlaybackFailure) {
            playerComplaint.value = failure.reason
        }
    }
}
