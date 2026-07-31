package dataClasses

/**
 * Назначенное нажатие: клавиша и зажатые вместе с ней модификаторы.
 *
 * [code] — код клавиши в том виде, в каком его отдаёт платформа; на десктопе это код AWT
 * (`KeyEvent.VK_…`). Хранится числом, а не объектом Compose: назначения должен понимать не только
 * Compose, но и глобальный перехват через X11, который будет следующим шагом.
 */
data class Hotkey(
    val code: Int,
    val ctrl: Boolean,
    val alt: Boolean,
    val shift: Boolean,
)
