package platform

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import platform.interfaces.PanelSettings

/**
 * На десктопе этих настроек нет.
 *
 * Панель управления здесь ни на чём не лежит: фон один на всё окно, отделять её линией не от чего.
 * А картинка обложки живёт в квадрате пластинки и под панель не заходит — прозрачность ей нечего
 * открывать.
 */
class PanelSettingsImpl : PanelSettings {

    @Composable
    override fun Content(
        color: Color,
        divider: Boolean,
        transparent: Boolean,
        onDivider: (Boolean) -> Unit,
        onTransparent: (Boolean) -> Unit,
    ) = Unit
}
