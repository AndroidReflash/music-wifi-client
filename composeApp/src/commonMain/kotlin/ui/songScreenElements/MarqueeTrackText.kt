package ui.songScreenElements

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.LinearOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.offset
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.IntOffset
import kotlinx.coroutines.delay
import kotlin.math.roundToInt

@Composable
fun MarqueeTrackTextFromRightToLeft(
    text: String,
    modifier: Modifier,
    fontSize: TextUnit,
) {

    val scrollSpeed = 1.5 // регулирование скорости перемотки
    var boxWidthPx by remember { mutableStateOf(0f) }
    val scroll = remember { Animatable(0f) }
    val textMeasurer = rememberTextMeasurer()
    val textWidthPx = textMeasurer.measure(
        text = AnnotatedString(text),
        style = TextStyle(
            fontSize = fontSize,
            lineHeight = fontSize
        ),
        maxLines = 1
    ).size.width.toFloat() * 1.1f //1.1f это величина, на которую, похоже, ошибается textMeasurer
    val needsScroll = textWidthPx - boxWidthPx > 0f
    // Запускаем анимацию только после того, как получили реальные размеры
    LaunchedEffect(textWidthPx, boxWidthPx, text) {
        scroll.snapTo(0f)
        delay(1000)
        if (needsScroll && boxWidthPx > 0f) {
            val duration = (((textWidthPx / 50f) * 1000) / scrollSpeed).toInt()
            val boxDuration = (((boxWidthPx / 50f) * 1000) / scrollSpeed).toInt()
            scroll.animateTo(
                textWidthPx,
                animationSpec = tween(durationMillis = duration, easing = LinearEasing)
            )
            while (true) {
                scroll.snapTo(-boxWidthPx)
                scroll.animateTo(
                    textWidthPx,
                    animationSpec = tween(durationMillis = duration + boxDuration, easing = LinearEasing)
                )
            }
        }
    }

    Box(
        modifier
            .clipToBounds() // обрезаем текст по границам Box
            .onGloballyPositioned { coordinates ->
                boxWidthPx = coordinates.size.width.toFloat()
            },
        contentAlignment = Alignment.Center
    ) {
        Text(
            text,
            fontSize = fontSize,
            lineHeight = fontSize,
            color = Color.White,
            softWrap = false,
            maxLines = 1,
            overflow = TextOverflow.Visible,
            modifier = Modifier
                .offset { IntOffset(- scroll.value.roundToInt(), 0) }
        )
    }
}


@Composable
fun MarqueeTrackTextBounce(
    text: String,
    modifier: Modifier = Modifier,
    fontSize: TextUnit,
) {
    val scrollSpeed = 50f // Пикселей в секунду
    var boxWidthPx by remember { mutableStateOf(0f) }
    val scroll = remember { Animatable(0f) }

    val textMeasurer = rememberTextMeasurer()
    val textStyle = TextStyle(
        fontSize = fontSize,
        lineHeight = fontSize,
    )

    val textLayoutResult = remember(text, fontSize) {
        textMeasurer.measure(AnnotatedString(text), textStyle)
    }
    val textWidthPx = textLayoutResult.size.width.toFloat()

    // Максимальное смещение — это то, насколько текст шире контейнера
    val maxScroll = (textWidthPx - boxWidthPx).coerceAtLeast(0f)
    val needsScroll = maxScroll > 0f

    LaunchedEffect(text, boxWidthPx) {
        if (needsScroll && boxWidthPx > 0f) {
            val duration = (maxScroll / scrollSpeed * 1000).toInt()

            while (true) {
                delay(2000) // Пауза перед началом движения

                // Едем влево (показываем правый край)
                scroll.animateTo(
                    targetValue = maxScroll,
                    animationSpec = tween(duration, easing = LinearOutSlowInEasing)
                )

                delay(2000) // Пауза на правом краю

                // Едем вправо (возвращаемся к началу)
                scroll.animateTo(
                    targetValue = 0f,
                    animationSpec = tween(duration, easing = LinearOutSlowInEasing)
                )
            }
        } else {
            scroll.snapTo(0f)
        }
    }

    Box(
        modifier = modifier
            .clipToBounds()
            .onGloballyPositioned { coordinates ->
                boxWidthPx = coordinates.size.width.toFloat()
            },
        contentAlignment = Alignment.Center
    ) {
        Text(
            text = text,
            style = textStyle,
            color = Color.White,
            softWrap = false,
            maxLines = 1,
            overflow = TextOverflow.Visible,
            modifier = Modifier
                .offset { IntOffset(-scroll.value.roundToInt(), 0) }
        )
    }
}