package constants

/** Величины плеера, у которых тип `Long`. */
sealed class PlayerConstantsLongs(val value: Long) {

    /**
     * Как часто спрашивать позицию, пока дорожка играет. Rust считает её в момент опроса,
     * событием она не приходит, поэтому ползунок движется ровно с этим шагом.
     */
    data object PositionPollMs : PlayerConstantsLongs(200)

    /** Шаг перемотки горячей клавишей. */
    data object SeekStepMs : PlayerConstantsLongs(5_000)

    /**
     * Глобальные клавиши: сколько держать перемотку или громкость, прежде чем они начнут
     * повторяться. В окне повторы присылает система со своими задержками; глобальный источник их
     * не присылает — портал Wayland сообщает только нажатие и отпускание, — и повторяет таймер.
     */
    data object GlobalRepeatDelayMs : PlayerConstantsLongs(500)

    /**
     * Глобальные клавиши: шаг повтора. Длиннее [PositionPollMs] ради перемотки: каждый её повтор
     * считает от места в снимке, и к повтору снимок должен успеть обновиться после прошлого сдвига.
     */
    data object GlobalRepeatIntervalMs : PlayerConstantsLongs(250)
}
