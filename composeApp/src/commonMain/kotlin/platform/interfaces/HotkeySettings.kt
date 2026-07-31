package platform.interfaces

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import dataClasses.Hotkey
import enums.HotkeyAction

/**
 * Горячие клавиши в окне настроек.
 *
 * Платформенная вещь: клавиатура есть у десктопа, а у телефона нет, поэтому телефонная половина не
 * рисует ничего. Живёт здесь по правилу шаблона: интерфейс с `@Composable`-методом — замена
 * `expect @Composable fun`, которой в Compose нет.
 */
interface HotkeySettings {

    /**
     * [hotkeys] — назначенные нажатия; неназначенных действий в словаре нет. [onAssign] назначает
     * нажатие действию, `null` снимает назначение. [global] — работают ли они вне окна.
     */
    @Composable
    fun Content(
        color: Color,
        hotkeys: Map<HotkeyAction, Hotkey>,
        onAssign: (HotkeyAction, Hotkey?) -> Unit,
        global: Boolean,
        onGlobal: (Boolean) -> Unit,
    )
}
