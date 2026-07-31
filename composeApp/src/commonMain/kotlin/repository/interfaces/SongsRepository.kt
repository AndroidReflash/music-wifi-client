package repository.interfaces

import androidx.compose.ui.graphics.ImageBitmap
import dataClasses.TrackInfo
import kotlinx.coroutines.flow.StateFlow

interface SongsRepository {
    fun play(trackInfo: TrackInfo)

    /**
     * Играть или стоять — приказ, а не тумблер. Так говорит уведомление и так спрашивает
     * media3: у неё на кнопке два разных состояния и она сама решает, какое прислать.
     */
    fun setPlaying(playing: Boolean)

    /**
     * Переключить: играет — остановить, стоит — играть. Для гарнитуры, у которой одна
     * кнопка и одна и та же команда на каждое нажатие.
     *
     * Состояние читается прямо у плеера в тот же миг, когда по нему принимается решение,
     * а не берётся из [isPlaying]: тот поток — пересказ последнего снимка, и между
     * пересказом и нажатием состояние могло смениться.
     */
    fun togglePlaying()

    /**
     * То же, что кнопка «играть» на экране: играющая дорожка встаёт на паузу или продолжается, а
     * если не выбрано ничего — включается случайная из списка.
     */
    fun playOrPause()
    fun nextTrack()
    fun prevTrack()
    fun seekTo(percentage: Float)

    /** Сдвиг от нынешнего места на [deltaMs], назад — отрицательным; за края дорожки не выходит. */
    fun seekBy(deltaMs: Long)
    fun shuffle()
    fun repeat()
    val isPlaying : StateFlow<Boolean>
    val currentPlay : StateFlow<TrackInfo?>
    val equalizerArray : StateFlow<List<Double>>

    /**
     * Последняя беда воспроизведения, словами для человека. Пустая строка означает, что
     * жаловаться не на что. Без неё отказ плеера некуда деть: команды уходят в свой поток
     * и возврата не ждут.
     */
    val playbackError: StateFlow<String>

    /**
     * Имена устройств вывода, на которые можно отправить звук. Пусто, пока не спросили:
     * перечисление стоит времени, поэтому оно делается по просьбе — [refreshOutputDevices].
     */
    val outputDevices: StateFlow<List<String>>

    /** Спрашивает список устройств заново. Звать при открытии настроек. */
    fun refreshOutputDevices()

    /**
     * Картинка обложки играющей дорожки, байтами из файла. `null` — обложки нет: её не прописали,
     * дорожка с сервера или не играет ничего.
     *
     * Байтами — для уведомления: оно само разбирает картинку. Экрану — [coverImage].
     */
    val cover: StateFlow<ByteArray?>

    /** Центры полос эквалайзера, Гц, от низкой к высокой — подписи ползунков. */
    val equalizerBands: List<Float>

    /** Предел ползунка эквалайзера, дБ, в обе стороны. */
    val equalizerMaxDb: Float

    /**
     * Усиления полос эквалайзера, дБ, ровно по одному на каждую из [equalizerBands]: полоса,
     * которую не трогали, стоит на нуле. Это и отдаётся плееру, и показывается ползунками.
     */
    val equalizerGains: StateFlow<List<Float>>

    /**
     * Та же обложка, уже разобранная в картинку. `null` — обложки нет или байты не разобрались.
     */
    val coverImage: StateFlow<ImageBitmap?>

    val currentTrackLength: StateFlow<Long>
    val currentPositionMs: StateFlow<Long>
    val shuffle: StateFlow<Boolean>
    val repeat: StateFlow<Boolean>
    /**
     * Список, из которого сейчас играют: свои файлы или отданный сервером. Он же —
     * очередь, отданная плееру, и он же лента для медиасессии.
     */
    val playlist: StateFlow<List<TrackInfo>>

    val songListOffline: StateFlow<List<TrackInfo>>
    val songListOnline: StateFlow<List<TrackInfo>>
    val isLocal : StateFlow<Boolean>
    fun onTrackSelectedManually(index: Int)

    /**
     * Пользователь сам взялся за интерфейс: автоплей, если он ещё ждёт списка, больше не нужен.
     * Дождавшийся или выключенный — ничего не меняется.
     */
    fun stopAutoplay()
}