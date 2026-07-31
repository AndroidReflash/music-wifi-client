package enums

/** Что плеер делает прямо сейчас. */
enum class PlaybackState {
    /** Звука нет: ещё не начинали, остановили или кончилась очередь. */
    IDLE,
    PLAYING,
    PAUSED,
}
