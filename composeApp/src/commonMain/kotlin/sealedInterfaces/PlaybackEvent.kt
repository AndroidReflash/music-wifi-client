package sealedInterfaces

import enums.PlaybackState

/** О чём плеер сообщает сам, не дожидаясь опроса. */
sealed interface PlaybackEvent {

    data class StateChanged(val state: PlaybackState) : PlaybackEvent

    /** Пошла дорожка с таким номером в списке. */
    data class TrackStarted(val index: Int) : PlaybackEvent

    /** Дорожка доиграна до конца — включая то, что оставалось в буфере устройства. */
    data object TrackFinished : PlaybackEvent

    /** Очередь кончилась: играть дальше нечего. */
    data object QueueFinished : PlaybackEvent

    /** Дорожка не открылась или оборвалась на полпути. */
    data class Failed(val failure: PlaybackFailure) : PlaybackEvent

    /**
     * Позиция сменилась скачком, а не ходом воспроизведения: перемотка, пауза, смена
     * устройства вывода. Само место события не несёт — оно в снимке, и читать его надо
     * оттуда, иначе у одной величины стало бы два источника.
     */
    data object PositionChanged : PlaybackEvent
}
