package platform

import constants.PlayerConstantsLongs
import dataClasses.Hotkey
import enums.HotkeyAction
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch
import platform.global_gestures.GestureListener
import platform.global_gestures.GlobalGestures
import platform.interfaces.GlobalHotkeys
import platform.interfaces.HotkeyActions
import repository.interfaces.SettingsRepository
import sealedInterfaces.GlobalHotkeysState

/**
 * Горячие клавиши вне окна. Пока настройка включена, держит источник нажатий под этот сеанс
 * ([GlobalGestures.forSession]) и отдаёт ему назначения при каждой их смене; выключили — закрывает.
 * Источник открывается только при включении: кто глобальные клавиши не включал, к дисплейному
 * серверу отдельно не подключается.
 *
 * Повтор при удержании (перемотка, громкость) — здесь, а не у источника: портал Wayland повторов
 * не присылает, только нажатие и отпускание, поэтому и X11 сообщает нажатие один раз.
 */
class GlobalHotkeysImpl(
    settings: SettingsRepository,
    private val actions: HotkeyActions,
    private val scope: CoroutineScope,
) : GlobalHotkeys {

    private val _state = MutableStateFlow<GlobalHotkeysState>(GlobalHotkeysState.Off)
    override val state: StateFlow<GlobalHotkeysState> = _state

    /**
     * Источник, пока включено; `null` — выключено. Трогается только из [follow], а тот зовётся
     * сборщиком настроек по одному.
     */
    private var gestures: GlobalGestures? = null

    /**
     * Действия, которые повторяются, пока держат клавишу. Под своим замком: нажатия приходят из
     * потока источника, а смена настроек — из сборщика.
     */
    private val repeating = mutableMapOf<HotkeyAction, Job>()

    private val listener = object : GestureListener {

        override fun onPress(action: HotkeyAction) {
            if (!action.repeats) {
                actions.perform(action)
                return
            }
            synchronized(repeating) {
                if (action !in repeating) {
                    repeating[action] = scope.launch { repeatWhileHeld(action) }
                }
            }
        }

        override fun onRelease(action: HotkeyAction) {
            synchronized(repeating) { repeating.remove(action) }?.cancel()
        }

        override fun onGrabbed(rejected: Set<HotkeyAction>) {
            _state.value = GlobalHotkeysState.Active(rejected)
        }

        override fun onUnavailable(reason: String) {
            _state.value = GlobalHotkeysState.Unavailable(reason)
        }
    }

    init {
        scope.launch {
            combine(settings.hotkeysGlobal, settings.hotkeys) { global, hotkeys ->
                hotkeys.takeIf { global }
            }.collect { assigned -> follow(assigned) }
        }
    }

    /** [assigned] — что перехватывать; `null` — глобальные клавиши выключены. */
    private fun follow(assigned: Map<HotkeyAction, Hotkey>?) {
        // Перехват меняется, и отпускание клавиши, зажатой под прежним, может не прийти вовсе.
        stopRepeats()
        if (assigned == null) {
            gestures?.close()
            gestures = null
            _state.value = GlobalHotkeysState.Off
            return
        }
        val open = gestures ?: GlobalGestures.forSession(listener).also { gestures = it }
        open.grab(assigned)
    }

    /** Первое срабатывание сразу, дальше — пока корутину не отменит отпускание. */
    private suspend fun repeatWhileHeld(action: HotkeyAction) {
        actions.perform(action)
        delay(PlayerConstantsLongs.GlobalRepeatDelayMs.value)
        while (true) {
            actions.perform(action)
            delay(PlayerConstantsLongs.GlobalRepeatIntervalMs.value)
        }
    }

    private fun stopRepeats() {
        synchronized(repeating) {
            repeating.values.forEach { job -> job.cancel() }
            repeating.clear()
        }
    }
}
