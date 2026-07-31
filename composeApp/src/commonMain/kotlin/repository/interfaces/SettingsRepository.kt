package repository.interfaces

import androidx.compose.ui.graphics.Color
import dataClasses.Hotkey
import enums.HotkeyAction
import kotlinx.coroutines.flow.StateFlow

interface SettingsRepository {
    val serverAddress: StateFlow<String>
    val isPlayingLocal: StateFlow<Boolean>
    val folderAddress: StateFlow<String>
    val interfaceColor: StateFlow<Color>

    /** Показывать ли полосы спектра. Скрывается и показывается отдельно от [showCover]. */
    val showVisualizer: StateFlow<Boolean>

    /** Показывать ли пластинку. Скрывается и показывается отдельно от [showVisualizer]. */
    val showCover: StateFlow<Boolean>

    /** Показывать ли картинку обложки из метаданных дорожки. */
    val showCoverImage: StateFlow<Boolean>

    /** Рисовать ли спектр кольцом. Иначе полосами. */
    val visualizerRing: StateFlow<Boolean>

    /** Рисовать ли пластинку уменьшенной — такой, чтобы она помещалась внутри кольца спектра. */
    val coverSmall: StateFlow<Boolean>

    /**
     * Рисовать ли линии, отделяющие список от полос — верхней с кнопками и нижней с управлением.
     * Обе от одной настройки: это одна черта оформления, а не две. Только телефон.
     */
    val panelDivider: StateFlow<Boolean>

    /**
     * Пускать ли картинку обложки под панель управления. Тогда она занимает весь экран, кроме
     * системных панелей. Только телефон.
     */
    val panelTransparent: StateFlow<Boolean>

    /** Куда выводить звук. Пустая строка — то устройство, которое система считает основным. */
    val outputDevice: StateFlow<String>

    /** Громкость самого приложения, от 0 до 1. Системную оно не трогает. */
    val volume: StateFlow<Float>

    /**
     * Усиления полос эквалайзера, дБ, от низкой полосы к высокой — как записаны. Полос, которых
     * не трогали, в конце списка может не быть: сколько их всего, знает плеер, а не настройки.
     */
    val equalizer: StateFlow<List<Float>>

    /** Горячие клавиши, только десктоп: действие → нажатие. Неназначенных действий в словаре нет. */
    val hotkeys: StateFlow<Map<HotkeyAction, Hotkey>>

    /**
     * Работают ли горячие клавиши вне окна — перехватываются у всей системы. Это желание, а не
     * факт: работает ли перехват на самом деле, знает тот, кто перехватывает.
     */
    val hotkeysGlobal: StateFlow<Boolean>

    /**
     * Перемешивание и повтор одной дорожки, как их оставили: переживают перезапуск. Это то, что
     * отдаётся плееру, а не то, что показывает кнопка, — кнопка показывает режим самого плеера.
     */
    val shuffle: StateFlow<Boolean>
    val repeat: StateFlow<Boolean>

    /** Начинать ли при запуске случайную дорожку, как только придёт список. */
    val autoplay: StateFlow<Boolean>
    suspend fun setServerAddress(address: String)
    suspend fun setOnlineToggle(value: Boolean)
    suspend fun setFolderAddress(address: String)
    suspend fun setThemeColor(color: Color)
    suspend fun setShowVisualizer(value: Boolean)
    suspend fun setShowCover(value: Boolean)
    suspend fun setOutputDevice(name: String)
    suspend fun setVolume(volume: Float)

    /**
     * Сдвигает громкость на [delta] от той, что записана, в пределах от 0 до 1. Чтение и запись —
     * одной правкой файла: удержание клавиши шлёт сдвиги чаще, чем обновляется [volume], и сдвиг
     * от его значения терял бы предыдущие.
     */
    suspend fun changeVolumeBy(delta: Float)

    /**
     * Записывает усиление одной полосы, дБ. Чтение и запись — одной правкой файла: две полосы,
     * сдвинутые подряд, иначе могли бы прочитать один и тот же прежний список, и вторая запись
     * стёрла бы первую.
     */
    suspend fun setEqualizerBand(band: Int, gainDb: Float)

    /** Все полосы эквалайзера на ноль — звук как без него. */
    suspend fun resetEqualizer()
    suspend fun setVisualizerRing(value: Boolean)
    suspend fun setCoverSmall(value: Boolean)
    suspend fun setShowCoverImage(value: Boolean)
    suspend fun setPanelDivider(value: Boolean)
    suspend fun setPanelTransparent(value: Boolean)

    /**
     * Назначает нажатие действию, `null` снимает назначение. Нажатие, уже занятое другим действием,
     * переходит к этому, а у прежнего снимается.
     */
    suspend fun setHotkey(action: HotkeyAction, hotkey: Hotkey?)
    suspend fun setHotkeysGlobal(value: Boolean)

    /**
     * Переключают записанное значение одной правкой файла: два нажатия подряд, прочитав одно и
     * то же прежнее значение, записали бы одно и то же новое, и второе нажатие потерялось бы.
     */
    suspend fun toggleShuffle()
    suspend fun toggleRepeat()
    suspend fun setAutoplay(value: Boolean)
}
