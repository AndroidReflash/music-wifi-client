package platform.global_gestures.wayland

import dataClasses.Hotkey
import enums.HotkeyAction
import platform.global_gestures.GestureListener
import platform.global_gestures.GlobalGestures

/**
 * Wayland: не сделано. Путь — портал `org.freedesktop.portal.GlobalShortcuts` через D-Bus:
 * программа передаёт список действий с желаемыми сочетаниями, рабочий стол сам спрашивает
 * пользователя и присылает нажатия и отпускания действий. Отсюда и устройство [GlobalGestures] — в
 * действиях, а не в клавишах.
 *
 * Пока — честный отказ: переключатель в настройках покажет причину, а не будет делать вид, что
 * работает.
 */
class WaylandGestures(private val listener: GestureListener) : GlobalGestures {

    override fun grab(assigned: Map<HotkeyAction, Hotkey>) {
        listener.onUnavailable("На Wayland глобальные клавиши пока не поддерживаются")
    }

    override fun close() = Unit
}
