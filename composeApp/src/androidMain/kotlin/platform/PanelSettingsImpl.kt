package platform

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import platform.interfaces.PanelSettings

/**
 * Два переключателя рисованными значками, рядом с остальными: линии у полос и прозрачность нижней
 * панели.
 *
 * Значки того же размера, что и общие, и работают так же: включено — цветом темы, выключено —
 * серым.
 */
class PanelSettingsImpl : PanelSettings {

    @Composable
    override fun Content(
        color: Color,
        divider: Boolean,
        transparent: Boolean,
        onDivider: (Boolean) -> Unit,
        onTransparent: (Boolean) -> Unit,
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Canvas(
                Modifier.size(GLYPH_SIDE).clickable { onDivider(!divider) }
            ) {
                dividerGlyph(if (divider) color else Color.Gray)
            }
            Canvas(
                Modifier.size(GLYPH_SIDE).clickable { onTransparent(!transparent) }
            ) {
                transparencyGlyph(if (transparent) color else Color.Gray)
            }
        }
    }
}

/** Линия над панелью: рамка панели внизу и черта над ней. */
private fun DrawScope.dividerGlyph(color: Color) {
    val side = size.minDimension
    val thin = side * 0.07f
    val left = size.width * 0.14f
    val right = size.width * 0.86f

    drawLine(
        color = color,
        start = Offset(left, size.height * 0.38f),
        end = Offset(right, size.height * 0.38f),
        strokeWidth = thin * 1.5f,
        cap = StrokeCap.Round,
    )
    drawRect(
        color = color,
        topLeft = Offset(left, size.height * 0.52f),
        size = Size(right - left, size.height * 0.30f),
        style = Stroke(width = thin),
    )
}

/**
 * Прозрачность: рамка, а внутри клетки через одну — так её обозначают везде, где вообще
 * обозначают.
 */
private fun DrawScope.transparencyGlyph(color: Color) {
    val side = size.minDimension
    val thin = side * 0.07f
    val left = size.width * 0.16f
    val top = size.height * 0.16f
    val box = size.width * 0.68f
    val cell = box / 2

    drawRect(
        color = color,
        topLeft = Offset(left, top),
        size = Size(box, box),
        style = Stroke(width = thin),
    )
    drawRect(color = color, topLeft = Offset(left, top), size = Size(cell, cell))
    drawRect(
        color = color,
        topLeft = Offset(left + cell, top + cell),
        size = Size(cell, cell),
    )
}

/** Тот же размер, что у общих значков настроек: полторы клетки выбора темы. */
private val GLYPH_SIDE = 36.dp
