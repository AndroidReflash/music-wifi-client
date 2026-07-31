package sealedInterfaces

import enums.HotkeyAction

/** Что на самом деле с глобальными клавишами — в отличие от настройки, которая только желание. */
sealed interface GlobalHotkeysState {

    /** Выключены в настройках: перехвата нет, клавиши работают только в окне. */
    data object Off : GlobalHotkeysState

    /** Включены, но перехватывать нечем. [reason] показывается в настройках как есть. */
    data class Unavailable(val reason: String) : GlobalHotkeysState

    /**
     * Перехват работает. [rejected] — действия, чьё сочетание перехватить не удалось (занято
     * другой программой или клавиша источнику неизвестна): они срабатывают только в окне.
     */
    data class Active(val rejected: Set<HotkeyAction>) : GlobalHotkeysState
}
