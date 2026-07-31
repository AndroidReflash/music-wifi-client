package platform

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import dataClasses.Hotkey
import enums.HotkeyAction
import platform.interfaces.HotkeySettings

/** У телефона клавиатуры нет, и назначать нечего. */
class HotkeySettingsImpl : HotkeySettings {

    @Composable
    override fun Content(
        color: Color,
        hotkeys: Map<HotkeyAction, Hotkey>,
        onAssign: (HotkeyAction, Hotkey?) -> Unit,
        global: Boolean,
        onGlobal: (Boolean) -> Unit,
    ) = Unit
}
