package repository

import androidx.compose.ui.graphics.Color
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.longPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import dataClasses.Hotkey
import enums.HotkeyAction
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.runBlocking
import repository.interfaces.SettingsRepository

class SettingsRepositoryImpl(
    private val dataStore: DataStore<Preferences>,
    private val scope: CoroutineScope
): SettingsRepository {

    private object Keys {
        val SERVER_ADDRESS = stringPreferencesKey("server_address")
        val ONLINE_TOGGLE = booleanPreferencesKey("online_toggle")
        val OFFLINE_FOLDER_ADDRESS = stringPreferencesKey("folder_address")
        val THEME_COLOR = longPreferencesKey("theme_color")
        val SHOW_VISUALIZER = booleanPreferencesKey("show_visualizer")
        val SHOW_COVER = booleanPreferencesKey("show_cover")
        val OUTPUT_DEVICE = stringPreferencesKey("output_device")
        val VOLUME = floatPreferencesKey("volume")
        val EQUALIZER = stringPreferencesKey("equalizer")
        val VISUALIZER_RING = booleanPreferencesKey("visualizer_ring")
        val COVER_SMALL = booleanPreferencesKey("cover_small")
        val SHOW_COVER_IMAGE = booleanPreferencesKey("show_cover_image")
        val PANEL_DIVIDER = booleanPreferencesKey("panel_divider")
        val PANEL_TRANSPARENT = booleanPreferencesKey("panel_transparent")
        val HOTKEYS_GLOBAL = booleanPreferencesKey("hotkeys_global")
        val SHUFFLE = booleanPreferencesKey("shuffle")
        val REPEAT = booleanPreferencesKey("repeat")
        val AUTOPLAY = booleanPreferencesKey("autoplay")

        /** Своя строка на каждое действие: `hotkey_play_pause` и так далее. */
        fun hotkey(action: HotkeyAction) =
            stringPreferencesKey("hotkey_${action.name.lowercase()}")
    }

    /**
     * Файл настроек, прочитанный один раз при создании. С него начинается каждая настройка: иначе
     * до чтения файла поток отдавал бы значение по умолчанию, и его успевали бы применить — экран
     * рисовал бы умолчание, а плеер получал бы его приказом.
     *
     * Цена: чтение ждёт тот, кто создаёт репозиторий, а на телефоне это главный поток при запуске.
     * Файл — пара сотен байт.
     */
    private val stored: Preferences = runBlocking { dataStore.data.first() }

    /** Настройка: значение из файла сейчас и дальше — при каждой его перезаписи. */
    private fun <T> setting(read: (Preferences) -> T): StateFlow<T> =
        dataStore.data.map(read).stateIn(scope, SharingStarted.WhileSubscribed(5000), read(stored))

    override val serverAddress: StateFlow<String> = setting { it[Keys.SERVER_ADDRESS] ?: "" }

    override val folderAddress: StateFlow<String> =
        setting { it[Keys.OFFLINE_FOLDER_ADDRESS] ?: "" }

    override val isPlayingLocal: StateFlow<Boolean> = setting { it[Keys.ONLINE_TOGGLE] ?: false }

    override val interfaceColor: StateFlow<Color> = setting { preferences ->
        val colorValue = preferences[Keys.THEME_COLOR]
        if (colorValue != null) Color(colorValue.toULong()) else Color.Cyan
    }

    // Ничего не выбрано — показываем: пустые настройки должны давать то, что видно.
    override val showVisualizer: StateFlow<Boolean> = setting { it[Keys.SHOW_VISUALIZER] ?: true }

    override val showCover: StateFlow<Boolean> = setting { it[Keys.SHOW_COVER] ?: true }

    // Пусто — значит основное устройство системы: так оно и было до появления этой настройки.
    override val outputDevice: StateFlow<String> = setting { it[Keys.OUTPUT_DEVICE] ?: "" }

    // Ничего не выбрано — полная громкость: так было до появления этой настройки.
    override val volume: StateFlow<Float> = setting { it[Keys.VOLUME] ?: 1f }

    // Ничего не записано — пустой список, то есть все полосы на нуле: так звучало до эквалайзера.
    override val equalizer: StateFlow<List<Float>> =
        setting { decodeEqualizer(it[Keys.EQUALIZER]) }

    // Ничего не выбрано — спектр полосами и пластинка во весь квадрат: так было до появления
    // этих настроек.
    override val visualizerRing: StateFlow<Boolean> =
        setting { it[Keys.VISUALIZER_RING] ?: false }

    override val coverSmall: StateFlow<Boolean> = setting { it[Keys.COVER_SMALL] ?: false }

    override val showCoverImage: StateFlow<Boolean> =
        setting { it[Keys.SHOW_COVER_IMAGE] ?: true }

    // Ничего не выбрано — как было до появления этих настроек: линии нет, картинка под панель не
    // заходит.
    override val panelDivider: StateFlow<Boolean> = setting { it[Keys.PANEL_DIVIDER] ?: false }

    override val panelTransparent: StateFlow<Boolean> =
        setting { it[Keys.PANEL_TRANSPARENT] ?: false }

    // Ничего не назначено — ничего и не срабатывает.
    override val hotkeys: StateFlow<Map<HotkeyAction, Hotkey>> = setting { preferences ->
        HotkeyAction.entries.mapNotNull { action ->
            preferences[Keys.hotkey(action)]?.let { stored -> action to decodeHotkey(stored) }
        }.toMap()
    }

    // Ничего не выбрано — только в окне: перехват у всей системы включают сознательно.
    override val hotkeysGlobal: StateFlow<Boolean> = setting { it[Keys.HOTKEYS_GLOBAL] ?: false }

    // Ничего не записано — выключены: так плеер и стартовал до появления этих настроек.
    override val shuffle: StateFlow<Boolean> = setting { it[Keys.SHUFFLE] ?: false }

    override val repeat: StateFlow<Boolean> = setting { it[Keys.REPEAT] ?: false }

    // Ничего не выбрано — выключен: так было до появления этой настройки.
    override val autoplay: StateFlow<Boolean> = setting { it[Keys.AUTOPLAY] ?: false }

    override suspend fun setServerAddress(address: String) {
        dataStore.edit { preferences ->
            preferences[Keys.SERVER_ADDRESS] = address
        }
    }

    override suspend fun setOnlineToggle(value: Boolean){
        dataStore.edit { preferences ->
            preferences[Keys.ONLINE_TOGGLE] = value
        }
    }

    override suspend fun setFolderAddress(address: String) {
        dataStore.edit { preferences ->
            preferences[Keys.OFFLINE_FOLDER_ADDRESS] = address
        }
    }

    override suspend fun setThemeColor(color: Color) {
        dataStore.edit { preferences ->
            preferences[Keys.THEME_COLOR] = color.value.toLong()
        }
    }

    override suspend fun setShowVisualizer(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.SHOW_VISUALIZER] = value
        }
    }

    override suspend fun setShowCover(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.SHOW_COVER] = value
        }
    }

    override suspend fun setOutputDevice(name: String) {
        dataStore.edit { preferences ->
            preferences[Keys.OUTPUT_DEVICE] = name
        }
    }

    override suspend fun setVolume(volume: Float) {
        dataStore.edit { preferences ->
            preferences[Keys.VOLUME] = volume
        }
    }

    override suspend fun changeVolumeBy(delta: Float) {
        dataStore.edit { preferences ->
            // Ничего не записано — полная громкость, как у `volume`.
            val current = preferences[Keys.VOLUME] ?: 1f
            preferences[Keys.VOLUME] = (current + delta).coerceIn(0f, 1f)
        }
    }

    override suspend fun setEqualizerBand(band: Int, gainDb: Float) {
        dataStore.edit { preferences ->
            val stored = decodeEqualizer(preferences[Keys.EQUALIZER])
            // Полосы до этой, которых ещё не трогали, записываются нулями: список хранит место
            // полосы порядком, и пропуск сдвинул бы её.
            val gains = MutableList(maxOf(stored.size, band + 1)) { stored.getOrElse(it) { 0f } }
            gains[band] = gainDb
            preferences[Keys.EQUALIZER] = gains.joinToString(",")
        }
    }

    // Запись снимается целиком, а не заполняется нулями: пустая настройка и значит «все на нуле».
    override suspend fun resetEqualizer() {
        dataStore.edit { preferences ->
            preferences.remove(Keys.EQUALIZER)
        }
    }

    override suspend fun setVisualizerRing(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.VISUALIZER_RING] = value
        }
    }

    override suspend fun setCoverSmall(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.COVER_SMALL] = value
        }
    }

    override suspend fun setShowCoverImage(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.SHOW_COVER_IMAGE] = value
        }
    }

    override suspend fun setPanelDivider(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.PANEL_DIVIDER] = value
        }
    }

    override suspend fun setPanelTransparent(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.PANEL_TRANSPARENT] = value
        }
    }

    // Одной записью: снять нажатие с прежнего действия и отдать новому. Две записи подряд оставили
    // бы между ними миг, когда нажатие назначено обоим.
    override suspend fun setHotkey(action: HotkeyAction, hotkey: Hotkey?) {
        dataStore.edit { preferences ->
            if (hotkey == null) {
                preferences.remove(Keys.hotkey(action))
            } else {
                val stored = encodeHotkey(hotkey)
                HotkeyAction.entries
                    .filter { other -> preferences[Keys.hotkey(other)] == stored }
                    .forEach { other -> preferences.remove(Keys.hotkey(other)) }
                preferences[Keys.hotkey(action)] = stored
            }
        }
    }

    override suspend fun setHotkeysGlobal(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.HOTKEYS_GLOBAL] = value
        }
    }

    override suspend fun toggleShuffle() {
        dataStore.edit { preferences ->
            preferences[Keys.SHUFFLE] = !(preferences[Keys.SHUFFLE] ?: false)
        }
    }

    override suspend fun toggleRepeat() {
        dataStore.edit { preferences ->
            preferences[Keys.REPEAT] = !(preferences[Keys.REPEAT] ?: false)
        }
    }

    override suspend fun setAutoplay(value: Boolean) {
        dataStore.edit { preferences ->
            preferences[Keys.AUTOPLAY] = value
        }
    }
}

/**
 * Нажатие строкой для файла настроек: `ctrl+alt+shift+<код>`, модификаторы только зажатые и всегда
 * в этом порядке. Порядок постоянный, поэтому одинаковые нажатия дают одинаковые строки, и занятое
 * нажатие ищется простым сравнением строк.
 */
private fun encodeHotkey(hotkey: Hotkey): String =
    listOfNotNull(
        "ctrl".takeIf { hotkey.ctrl },
        "alt".takeIf { hotkey.alt },
        "shift".takeIf { hotkey.shift },
        hotkey.code.toString(),
    ).joinToString("+")

/** Усиления полос из файла настроек: числа через запятую. `null` и пустая строка — не трогали. */
private fun decodeEqualizer(stored: String?): List<Float> =
    if (stored.isNullOrEmpty()) emptyList() else stored.split(",").map { it.toFloat() }

private fun decodeHotkey(stored: String): Hotkey {
    val parts = stored.split("+")
    return Hotkey(
        code = parts.last().toInt(),
        ctrl = "ctrl" in parts,
        alt = "alt" in parts,
        shift = "shift" in parts,
    )
}
