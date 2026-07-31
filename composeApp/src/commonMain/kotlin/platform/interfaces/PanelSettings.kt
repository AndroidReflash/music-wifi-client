package platform.interfaces

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

/**
 * Настройки полос: линии, отделяющие их от списка, и то, пускать ли картинку обложки под нижнюю.
 *
 * Платформенная вещь, и на этот раз рисует телефон, а десктоп молчит: там панель ни на чём не лежит
 * — фон один на всё окно, — а картинка живёт в квадрате пластинки и под панель не заходит по
 * устройству расклада. Настройка, которая ничего не меняет, хуже отсутствующей.
 */
interface PanelSettings {

    @Composable
    fun Content(
        color: Color,
        divider: Boolean,
        transparent: Boolean,
        onDivider: (Boolean) -> Unit,
        onTransparent: (Boolean) -> Unit,
    )
}
