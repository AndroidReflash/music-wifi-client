package platform.global_gestures.x11

import constants.XcbConstantsInts
import dataClasses.Hotkey
import enums.HotkeyAction
import java.util.concurrent.ConcurrentHashMap
import platform.global_gestures.GestureListener
import platform.global_gestures.GlobalGestures
import sealedInterfaces.XcbEvent

/**
 * X11: перехват через `GrabKey` на корневом окне — X отдаёт захваченное сочетание нам, в каком бы
 * окне ни был фокус.
 *
 * Подключение открывается при создании; не открылось — каждый [grab] отвечает причиной через
 * [GestureListener.onUnavailable]. События читает свой поток; он же зовёт [GestureListener.onPress]
 * и [GestureListener.onRelease].
 */
class X11Gestures(private val listener: GestureListener) : GlobalGestures {

    private val opened: Result<XcbConnection> = runCatching { XcbConnection.open() }

    /**
     * Перехваченное: keycode и модификаторы → действие. Пишется в [grab], читается потоком
     * событий.
     */
    @Volatile
    private var grabbed: Map<Pair<Int, Int>, HotkeyAction> = emptyMap()

    /**
     * Зажатые перехваченные клавиши: keycode → действие. Нажатие клавиши, которая уже здесь, —
     * автоповтор. Чистится при каждом [grab]: отпускание клавиши, зажатой под прежним захватом,
     * может уйти другому окну, и следующее её нажатие сочлось бы автоповтором.
     */
    private val held = ConcurrentHashMap<Int, HotkeyAction>()

    @Volatile
    private var closing = false

    private val events: Thread? = opened.getOrNull()?.let { connection ->
        Thread({ readEvents(connection) }, "x11-gestures").apply {
            // Выход из приложения не ждёт этот поток: захваты X снимет сам, когда закроется
            // подключение вместе с процессом.
            isDaemon = true
            start()
        }
    }

    override fun grab(assigned: Map<HotkeyAction, Hotkey>) {
        val connection = opened.getOrElse { failure ->
            listener.onUnavailable("X11: ${failure.message}")
            return
        }
        connection.ungrabAll()
        held.clear()
        val taken = mutableMapOf<Pair<Int, Int>, HotkeyAction>()
        val rejected = mutableSetOf<HotkeyAction>()
        // Caps Lock и Num Lock для X — модификаторы: без этих вариантов сочетание молчало бы, пока
        // любой из них включён. Num Lock, не посаженный ни на один модификатор, даёт 0, и
        // одинаковые варианты схлопываются.
        val lock = XcbConstantsInts.LockMask.value
        val numLock = connection.numLockMask
        val locks = listOf(0, lock, numLock, lock or numLock).distinct()
        assigned.forEach { (action, hotkey) ->
            val keycode = keysymOf(hotkey.code)?.let(connection::keycodeOf)
            val modifiers = modifiersOf(hotkey, connection.altMask)
            if (keycode != null && modifiers != null &&
                connection.grab(keycode, locks.map { extra -> modifiers or extra })
            ) {
                taken[keycode to modifiers] = action
            } else {
                rejected += action
            }
        }
        grabbed = taken
        listener.onGrabbed(rejected)
    }

    override fun close() {
        val connection = opened.getOrNull() ?: return
        closing = true
        connection.ungrabAll()
        connection.wake()
        events?.join()
        connection.disconnect()
    }

    private fun readEvents(connection: XcbConnection) {
        while (true) {
            when (val event = connection.nextEvent()) {
                null -> {
                    if (!closing) {
                        listener.onUnavailable("X11: подключение к X-серверу оборвалось")
                    }
                    return
                }
                XcbEvent.Wake -> return
                is XcbEvent.KeyPress -> if (!closing) press(event, connection)
                is XcbEvent.KeyRelease -> if (!closing) release(event)
                XcbEvent.Other -> Unit
            }
        }
    }

    /**
     * Пока захваченная клавиша зажата, X отдаёт нам всю клавиатуру, поэтому сюда приходят и чужие
     * нажатия — их в [grabbed] нет, и они пропускаются. Из маски берутся только Shift, Ctrl и Alt:
     * там же лежат Caps Lock, Num Lock, кнопки мыши и номер раскладки.
     */
    private fun press(event: XcbEvent.KeyPress, connection: XcbConnection) {
        val relevant = XcbConstantsInts.ShiftMask.value or
            XcbConstantsInts.ControlMask.value or
            (connection.altMask ?: 0)
        val action = grabbed[event.keycode to (event.state and relevant)] ?: return
        if (held.putIfAbsent(event.keycode, action) == null) {
            listener.onPress(action)
        }
    }

    private fun release(event: XcbEvent.KeyRelease) {
        held.remove(event.keycode)?.let(listener::onRelease)
    }
}

/**
 * Модификаторы назначения маской X. `null` — в назначении Alt, а Alt не посажен ни на один
 * модификатор: без него захват поймал бы сочетание без Alt, то есть не то.
 */
private fun modifiersOf(hotkey: Hotkey, altMask: Int?): Int? {
    val alt = if (hotkey.alt) altMask ?: return null else 0
    return alt or
        (if (hotkey.shift) XcbConstantsInts.ShiftMask.value else 0) or
        (if (hotkey.ctrl) XcbConstantsInts.ControlMask.value else 0)
}
