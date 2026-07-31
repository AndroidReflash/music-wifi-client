package dataClasses

import enums.PlaybackRepeat
import enums.PlaybackState

/**
 * Состояние плеера на момент опроса. Позиция считается в момент вызова, а не копится
 * событиями, поэтому снимок свежий и между событиями.
 */
data class PlaybackSnapshot(
    val state: PlaybackState,
    /** Сколько текущей дорожки уже прозвучало. */
    val positionMs: Long,
    /** Длительность текущей дорожки, если контейнер её объявил. */
    val durationMs: Long?,
    /** Номер текущей дорожки в списке — тот, который показывают пользователю. */
    val trackIndex: Int?,
    val trackCount: Int,
    /** Есть ли куда идти кнопками: уведомление спрашивает об этом, из списка не выводит. */
    val hasNext: Boolean,
    val hasPrevious: Boolean,
    val shuffle: Boolean,
    val repeat: PlaybackRepeat,
)
