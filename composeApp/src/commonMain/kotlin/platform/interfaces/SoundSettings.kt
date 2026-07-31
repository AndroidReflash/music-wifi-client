package platform.interfaces

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/**
 * Настройки звука в окне настроек: куда выводить и как громко.
 *
 * Платформенная вещь, а не общая. На Android маршрутом распоряжается сама система — подключённые
 * наушники она переносит на себя без нашего участия, — а громкость там системная, кнопками на
 * корпусе. Перечисление устройств на Android к тому же дорогое: каждое опрашивается через JNI.
 * Поэтому телефонная половина не рисует ничего и [onRefresh] не зовёт вовсе.
 *
 * Живёт здесь по правилу шаблона: интерфейс с `@Composable`-методом — замена `expect @Composable
 * fun`, которой в Compose нет.
 */
interface SoundSettings {

    /**
     * [devices] — имена, на которые звук выводить можно; [device] — выбранное, пустая строка
     * означает основное устройство системы. [onRefresh] спрашивает список заново: перечисление
     * стоит времени, поэтому зовётся оно из этой половины, когда она появляется на экране.
     *
     * [volume] — громкость приложения от 0 до 1.
     */
    @Composable
    fun Content(
        color: Color,
        devices: List<String>,
        device: String,
        volume: Float,
        onRefresh: () -> Unit,
        onPickDevice: (String) -> Unit,
        onVolume: (Float) -> Unit,
    )
}
