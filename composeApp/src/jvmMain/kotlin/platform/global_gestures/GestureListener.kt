package platform.global_gestures

import enums.HotkeyAction

/**
 * Что источник глобальных нажатий сообщает наверх. Всё — в действиях, а не в клавишах: на Wayland
 * клавиши программе не видны вовсе, портал присылает имя действия.
 *
 * Зовётся из потока источника. После [GlobalGestures.close] не зовётся.
 */
interface GestureListener {

    /**
     * Сочетание действия нажато. Автоповтор сюда не доходит: одно нажатие — один вызов, повторы
     * делает тот, кто слушает.
     */
    fun onPress(action: HotkeyAction)

    /** Сочетание отпущено. */
    fun onRelease(action: HotkeyAction)

    /**
     * Ответ на [GlobalGestures.grab]: перехват стоит, [rejected] — действия, чьё сочетание
     * перехватить не удалось: оно занято другой программой или клавиша источнику неизвестна.
     * Пустой набор — перехвачено всё.
     */
    fun onGrabbed(rejected: Set<HotkeyAction>)

    /**
     * Перехватывать нечем: сеанс не тот, сервер не пустил. [reason] показывается пользователю как
     * есть.
     */
    fun onUnavailable(reason: String)
}
