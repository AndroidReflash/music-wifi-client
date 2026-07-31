package ui.songScreenElements

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
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

/**
 * Пять переключателей рисованными значками: показывать ли спектр, пластинку и картинку обложки,
 * каким рисовать спектр и какого размера пластинку.
 *
 * Первые два — это «да или нет», и состояние видно цветом: включено — цветом темы, выключено —
 * серым. У вторых двух «нет» не бывает, у них два равноправных вида, поэтому состояние видно самим
 * значком: полосы или кольцо, стрелка вниз или вверх. Стрелка показывает, что сделает нажатие, а не
 * что сейчас.
 */
@Composable
fun SettingGlyphs(
    color: Color,
    showVisualizer: Boolean,
    showCover: Boolean,
    showCoverImage: Boolean,
    ring: Boolean,
    small: Boolean,
    onShowVisualizer: (Boolean) -> Unit,
    onShowCover: (Boolean) -> Unit,
    onShowCoverImage: (Boolean) -> Unit,
    onRing: (Boolean) -> Unit,
    onSmall: (Boolean) -> Unit,
) {
    Row(
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Canvas(glyph { onShowVisualizer(!showVisualizer) }) {
            bars(if (showVisualizer) color else Color.Gray)
        }
        Canvas(glyph { onShowCover(!showCover) }) {
            disc(if (showCover) color else Color.Gray, radius = 0.40f)
        }
        Canvas(glyph { onShowCoverImage(!showCoverImage) }) {
            picture(if (showCoverImage) color else Color.Gray)
        }
        Canvas(glyph { onRing(!ring) }) {
            if (ring) rays(color) else bars(color)
        }
        Canvas(glyph { onSmall(!small) }) {
            disc(color, radius = if (small) 0.20f else 0.32f, shift = -0.12f)
            arrow(color, up = small)
        }
    }
}

/** Общий размер и нажатие: полторы клетки выбора темы. */
private fun glyph(onClick: () -> Unit): Modifier = Modifier
    .size(36.dp)
    .clickable { onClick() }

/** Полосы спектра: пять столбиков разной высоты. */
private fun DrawScope.bars(color: Color) {
    val thickness = size.width * 0.1f
    val heights = listOf(0.35f, 0.6f, 0.9f, 0.5f, 0.3f)
    val places = listOf(0.15f, 0.325f, 0.5f, 0.675f, 0.85f)

    heights.forEachIndexed { index, share ->
        val x = size.width * places[index]
        val half = size.height * share / 2
        drawLine(
            color = color,
            start = Offset(x, size.height / 2 - half),
            end = Offset(x, size.height / 2 + half),
            strokeWidth = thickness,
            cap = StrokeCap.Round,
        )
    }
}

/** Кольцевой спектр: лучи по кругу, через один подлиннее. */
private fun DrawScope.rays(color: Color) {
    val side = size.minDimension
    val middle = Offset(size.width / 2, size.height / 2)
    val rays = 12

    for (index in 0 until rays) {
        val radians = index * (360f / rays) * PI.toFloat() / 180f
        val direction = Offset(cos(radians), sin(radians))
        val outer = if (index % 2 == 0) 0.46f else 0.34f
        drawLine(
            color = color,
            start = middle + direction * (side * 0.18f),
            end = middle + direction * (side * outer),
            strokeWidth = side * 0.07f,
            cap = StrokeCap.Round,
        )
    }
}

/** Картинка: рамка, гора и солнце — то, чем обычно обозначают изображение. */
private fun DrawScope.picture(color: Color) {
    val side = size.minDimension
    val thin = side * 0.07f
    val left = size.width * 0.14f
    val right = size.width * 0.86f
    val top = size.height * 0.20f
    val bottom = size.height * 0.80f

    drawRect(
        color = color,
        topLeft = Offset(left, top),
        size = Size(right - left, bottom - top),
        style = Stroke(width = thin),
    )
    drawCircle(
        color = color,
        radius = side * 0.07f,
        center = Offset(size.width * 0.34f, size.height * 0.36f),
    )
    // Гора: две прямые от основания к вершине и обратно, внутри рамки.
    drawLine(
        color = color,
        start = Offset(left + thin, bottom - thin),
        end = Offset(size.width * 0.52f, size.height * 0.44f),
        strokeWidth = thin,
        cap = StrokeCap.Round,
    )
    drawLine(
        color = color,
        start = Offset(size.width * 0.52f, size.height * 0.44f),
        end = Offset(right - thin, bottom - thin),
        strokeWidth = thin,
        cap = StrokeCap.Round,
    )
}

/** Пластинка: край и отверстие. [shift] сдвигает её по горизонтали долей стороны. */
private fun DrawScope.disc(color: Color, radius: Float, shift: Float = 0f) {
    val side = size.minDimension
    val middle = Offset(size.width / 2 + side * shift, size.height / 2)

    drawCircle(
        color = color,
        radius = side * radius,
        center = middle,
        style = Stroke(width = side * 0.07f),
    )
    drawCircle(color = color, radius = side * 0.06f, center = middle)
}

/** Стрелка справа: вверх — «увеличить», вниз — «уменьшить». */
private fun DrawScope.arrow(color: Color, up: Boolean) {
    val side = size.minDimension
    val x = size.width * 0.82f
    val top = size.height * 0.22f
    val bottom = size.height * 0.78f
    val thickness = side * 0.07f
    val wing = side * 0.12f
    val tip = if (up) top else bottom
    val toward = if (up) wing else -wing

    drawLine(
        color = color,
        start = Offset(x, top),
        end = Offset(x, bottom),
        strokeWidth = thickness,
        cap = StrokeCap.Round,
    )
    drawLine(
        color = color,
        start = Offset(x - wing, tip + toward),
        end = Offset(x, tip),
        strokeWidth = thickness,
        cap = StrokeCap.Round,
    )
    drawLine(
        color = color,
        start = Offset(x + wing, tip + toward),
        end = Offset(x, tip),
        strokeWidth = thickness,
        cap = StrokeCap.Round,
    )
}
