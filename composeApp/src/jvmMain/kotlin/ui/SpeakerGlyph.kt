package ui

import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp

/**
 * Динамик тонкими линиями: корпус прямыми отрезками, волны дугами. Кривых нет намеренно — их
 * пришлось бы задавать на глаз, и это было бы видно.
 *
 * [waves] — сколько волн. У ползунка громкости две; тот же динамик с одной и тремя — значки «тише»
 * и «громче» в горячих клавишах.
 */
@Composable
fun SpeakerGlyph(color: Color, modifier: Modifier = Modifier, waves: Int = 2) {
    Canvas(modifier) {
        val thin = 1.dp.toPx()
        val w = size.width
        val h = size.height

        val body = Path().apply {
            moveTo(w * 0.08f, h * 0.38f)
            lineTo(w * 0.30f, h * 0.38f)
            lineTo(w * 0.55f, h * 0.15f)
            lineTo(w * 0.55f, h * 0.85f)
            lineTo(w * 0.30f, h * 0.62f)
            lineTo(w * 0.08f, h * 0.62f)
            close()
        }
        drawPath(path = body, color = color, style = Stroke(width = thin))

        // Волны идут от раструба вправо: дуги по 90°, серединой на три часа, через 0,12 ширины.
        // Первая — на 0,22 ширины, если последняя при этом не выходит за 0,40: дальше она
        // упёрлась бы в край. Иначе весь веер сдвигается к раструбу.
        val first = minOf(0.22f, 0.40f - 0.12f * (waves - 1))
        repeat(waves) { wave ->
            val radius = w * (first + 0.12f * wave)
            val center = Offset(w * 0.55f, h * 0.5f)
            drawArc(
                color = color,
                startAngle = -45f,
                sweepAngle = 90f,
                useCenter = false,
                topLeft = Offset(center.x - radius, center.y - radius),
                size = Size(radius * 2, radius * 2),
                style = Stroke(width = thin),
            )
        }
    }
}
