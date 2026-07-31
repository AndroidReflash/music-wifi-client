package platform.global_gestures

import dataClasses.Hotkey
import enums.HotkeyAction
import platform.global_gestures.wayland.WaylandGestures
import platform.global_gestures.x11.X11Gestures

/**
 * Источник глобальных нажатий — одна реализация на дисплейный сервер. Что и как захватывать,
 * решает реализация; наверх она сообщает через [GestureListener].
 */
interface GlobalGestures {

    /**
     * Перехватить эти назначения вместо прежних. Пустой словарь снимает всё. Ответ приходит в
     * [GestureListener.onGrabbed] или [GestureListener.onUnavailable] — не обязательно до возврата:
     * портал Wayland отвечает позже, отдельным сообщением.
     */
    fun grab(assigned: Map<HotkeyAction, Hotkey>)

    /** Снять перехват и отключиться. Дальше источник не используется. */
    fun close()

    companion object {

        /**
         * Источник для сеанса, в котором запущено приложение. Выбирается по окружению, а не
         * настройкой и не сборкой: сеанс выбирают при входе в систему, и одна установленная
         * программа должна работать в любом.
         *
         * `WAYLAND_DISPLAY` — имя сокета композитора — проверяется первым. Под Wayland задан и
         * `DISPLAY`: его ставит XWayland для программ, умеющих только X11, — в том числе для нашего
         * окна. Захват через X11 там не упадёт, но нажатия будут приходить, только пока в фокусе
         * окно XWayland, — то есть глобальные клавиши молча работали бы лишь там, где работают и
         * без них.
         */
        fun forSession(listener: GestureListener): GlobalGestures =
            if (System.getenv("WAYLAND_DISPLAY") != null) {
                WaylandGestures(listener)
            } else {
                X11Gestures(listener)
            }
    }
}
