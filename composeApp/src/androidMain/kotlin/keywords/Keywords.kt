package keywords

sealed class Keywords (val value: String) {
    data object PlayerId: Keywords("player")
    data object CloseAction: Keywords("close_action")

    /** Тег журнала. Тот же, что у Rust-части: одна строка `adb logcat -s musicwifi`. */
    data object LogTag: Keywords("musicwifi")
}