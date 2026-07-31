package ui.songScreenElements

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp

/**
 * Надпись с чёрной обводкой: снизу контур, сверху заливка.
 *
 * Нужна там, где под надписью может оказаться картинка обложки. Одной заливки мало — на светлой
 * картинке она сливается; одного контура тоже — полые буквы плохо читаются.
 *
 * Модификаторов два, и это не прихоть. [modifier] решает, как надпись стоит у родителя (вес в
 * строке, например), а [textModifier] достаётся обоим слоям и решает, как ведёт себя сам текст
 * (отступы, ширина). Слои обязаны получить одно и то же, иначе они разъедутся.
 */
@Composable
fun OutlinedText(
    text: String,
    color: Color,
    modifier: Modifier = Modifier,
    textModifier: Modifier = Modifier,
    fontSize: TextUnit = TextUnit.Unspecified,
    lineHeight: TextUnit = TextUnit.Unspecified,
    textAlign: TextAlign? = null,
    maxLines: Int = Int.MAX_VALUE,
    overflow: TextOverflow = TextOverflow.Clip,
) {
    Box(modifier) {
        Text(
            text = text,
            modifier = textModifier,
            color = Color.Black,
            fontSize = fontSize,
            lineHeight = lineHeight,
            textAlign = textAlign,
            maxLines = maxLines,
            overflow = overflow,
            style = TextStyle(drawStyle = Stroke(width = OUTLINE_WIDTH), fontSize = fontSize),
        )
        Text(
            text = text,
            modifier = textModifier,
            color = color,
            fontSize = fontSize,
            lineHeight = lineHeight,
            textAlign = textAlign,
            maxLines = maxLines,
            overflow = overflow,
        )
    }
}

/**
 * Значок с чёрной обводкой.
 *
 * У растрового значка контура нет, поэтому обводка делается иначе, чем у букв: чёрная копия
 * рисуется восемь раз со сдвигом на точку во все стороны, а цветная — сверху. Восемь, а не четыре:
 * по диагоналям остаются щели на тонких линиях.
 *
 * Размер и нажатие приходят в [modifier] — так же, как их задавали одиночному значку.
 */
@Composable
fun OutlinedIcon(
    painter: Painter,
    contentDescription: String?,
    tint: Color,
    modifier: Modifier = Modifier,
) {
    Box(modifier) {
        OUTLINE_SHIFTS.forEach { (dx, dy) ->
            Icon(
                painter = painter,
                contentDescription = null,
                tint = Color.Black,
                modifier = Modifier.matchParentSize().offset(x = dx, y = dy),
            )
        }
        Icon(
            painter = painter,
            contentDescription = contentDescription,
            tint = tint,
            modifier = Modifier.matchParentSize(),
        )
    }
}

/**
 * Толщина обводки букв в точках холста. Контур рисуется по середине линии, то есть половина его
 * уходит внутрь; заливка сверху это закрывает.
 */
private const val OUTLINE_WIDTH = 3f

/** Куда сдвигать чёрные копии значка. */
private val OUTLINE_SHIFTS: List<Pair<Dp, Dp>> = listOf(
    (-1).dp to 0.dp,
    1.dp to 0.dp,
    0.dp to (-1).dp,
    0.dp to 1.dp,
    (-1).dp to (-1).dp,
    1.dp to (-1).dp,
    (-1).dp to 1.dp,
    1.dp to 1.dp,
)
