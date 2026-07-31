package platform

import androidx.compose.ui.input.key.KeyEvent
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.nativeKeyCode
import androidx.compose.ui.input.key.type
import dataClasses.Hotkey
import enums.HotkeyAction
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import platform.interfaces.HotkeyActions
import platform.interfaces.WindowHotkeys
import repository.interfaces.SettingsRepository

/**
 * Горячие клавиши окна.
 *
 * Нажатие приходит сюда из `onPreviewKeyEvent` окна — раньше, чем его получит элемент в фокусе.
 * Иначе назначенный пробел забирала бы кнопка, нажатая последней: на десктопе Compose при щелчке
 * отдаёт ей фокус, а кнопка в фокусе сама жмётся пробелом. Исключение — поле поиска: пока в нём
 * набирают текст ([setTextEntry]), нажатия принадлежат ему.
 *
 * Второй источник нажатий — глобальный перехват (`GlobalHotkeysImpl`); что делает действие, оба
 * берут у [HotkeyActions].
 */
class WindowHotkeysImpl(
    settings: SettingsRepository,
    private val actions: HotkeyActions,
    scope: CoroutineScope,
) : WindowHotkeys {

    /**
     * Назначения. Своя копия, а не `settings.hotkeys.value`: тот поток разделяется, только пока
     * его кто-то слушает, и без слушателя его значение перестаёт обновляться.
     */
    @Volatile
    private var assigned: Map<HotkeyAction, Hotkey> = settings.hotkeys.value

    /**
     * Зажатые клавиши. Автоповтор присылает нажатие снова, не отпуская клавишу, — по этому набору
     * его и отличают от нового нажатия. Трогается только из потока окна.
     */
    private val held = mutableSetOf<Int>()

    /** Курсор в поле поиска. Трогается только из потока окна. */
    private var textEntry = false

    init {
        scope.launch {
            settings.hotkeys.collect { hotkeys -> assigned = hotkeys }
        }
    }

    /**
     * Отпускание забирается вместе с нажатием: кнопка в фокусе жмётся на отпускании пробела, и
     * отданное ей отпускание без нажатия могло бы её нажать.
     */
    override fun onPreviewKeyEvent(event: KeyEvent): Boolean {
        val code = event.key.nativeKeyCode
        return when (event.type) {
            KeyEventType.KeyUp -> held.remove(code)
            KeyEventType.KeyDown -> !textEntry && press(
                Hotkey(
                    code = code,
                    ctrl = event.isCtrlPressed,
                    alt = event.isAltPressed,
                    shift = event.isShiftPressed,
                )
            )
            else -> false
        }
    }

    override fun setTextEntry(active: Boolean) {
        textEntry = active
    }

    /**
     * Отпускание клавиш придёт уже не окну, и без очистки следующее нажатие той же клавиши сочлось
     * бы автоповтором.
     */
    override fun releaseAll() {
        held.clear()
    }

    private fun press(pressed: Hotkey): Boolean {
        val action = assigned.entries.firstOrNull { (_, hotkey) -> hotkey == pressed }?.key
            ?: return false
        val repeated = !held.add(pressed.code)
        if (!repeated || action.repeats) {
            actions.perform(action)
        }
        return true
    }
}
