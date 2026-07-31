package sealedInterfaces

/** Событие X, разобранное из того, что отдал XCB. Нужные перехвату — отдельно, прочие — [Other]. */
sealed interface XcbEvent {

    /** [state] — модификаторы и кнопки, зажатые в момент нажатия, маской X. */
    data class KeyPress(val keycode: Int, val state: Int) : XcbEvent

    data class KeyRelease(val keycode: Int) : XcbEvent

    /** Подключение само себя разбудило: поток событий должен завершиться. */
    data object Wake : XcbEvent

    /** Всё остальное, в том числе ошибки непроверяемых запросов. */
    data object Other : XcbEvent
}
