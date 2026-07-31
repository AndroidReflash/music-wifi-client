package ui.songScreenElements

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp

/**
 * Рисованная пластинка тонкими линиями. Картинка обложки к ней отношения не имеет: у неё свой слой
 * и своё место, разное на телефоне и на десктопе, — [CoverImage].
 *
 * Пластинка нарисована окружностями и дугами, а не кривыми: кривые пришлось бы задавать на глаз,
 * и это было бы видно. Размер берётся от переданного модификатора и считается от меньшей его
 * стороны, поэтому квадрат задаёт тот, кто её ставит.
 *
 * Пока [isPlaying], дуги дорожек вращаются вокруг этикетки — пластинка как будто крутится. Стоит
 * воспроизведению встать, вращение замирает там, где застало: угол не сбрасывается.
 *
 * [small] рисует ту же пластинку уменьшенной — так, чтобы она помещалась внутри кольца спектра.
 * Уменьшается только рисунок: отведённое место остаётся прежним, иначе под ним поехало бы всё
 * остальное.
 */
@Composable
fun CoverArt(
    color: Color,
    isPlaying: Boolean,
    small: Boolean,
    modifier: Modifier = Modifier,
) {
    val share = if (small) SMALL_SHARE else 1f
    val spin = remember { mutableFloatStateOf(0f) }

    // Угол крутится по кадрам, а не переходом на фиксированный срок: переход пришлось бы либо
    // обрывать в начало на паузе, либо держать вечно. Здесь остановка — это отмена эффекта, и
    // угол остаётся тем, что был.
    LaunchedEffect(isPlaying) {
        if (!isPlaying) {
            return@LaunchedEffect
        }
        var previous = withFrameNanos { nanos -> nanos }
        while (true) {
            val now = withFrameNanos { nanos -> nanos }
            val seconds = (now - previous) / 1_000_000_000f
            previous = now
            spin.floatValue = (spin.floatValue + seconds * DEGREES_PER_SECOND) % 360f
        }
    }

    Box(modifier, contentAlignment = Alignment.Center) {
        Canvas(Modifier.fillMaxSize()) {
            val side = size.minDimension * share
            val thin = 1.dp.toPx()

            // Край, этикетка и отверстие — целыми окружностями.
            drawCircle(color = color, radius = side * 0.48f, style = Stroke(width = thin * 1.5f))
            drawCircle(color = color, radius = side * 0.16f, style = Stroke(width = thin * 1.5f))
            drawCircle(color = color, radius = side * 0.03f, style = Stroke(width = thin))

            // Три дорожки между этикеткой и краем — двумя дугами по 45° каждая: слева сверху и
            // справа снизу. У `drawArc` ноль градусов смотрит вправо, углы растут по часовой:
            // значит налево-вверх — это 225°, направо-вниз — 45°, и от каждой из этих середин
            // дуга отступает на половину своей ширины.
            // Угол читается внутри отрисовки, а не снаружи: тогда каждый кадр заново рисует, а
            // не пересобирает разметку.
            val turned = spin.floatValue
            listOf(0.28f, 0.34f, 0.40f).forEach { share ->
                val radius = side * share
                val corner = Offset(center.x - radius, center.y - radius)
                val box = Size(radius * 2, radius * 2)
                drawArc(
                    color = color,
                    startAngle = turned + 225f - SECTOR / 2,
                    sweepAngle = SECTOR,
                    useCenter = false,
                    topLeft = corner,
                    size = box,
                    style = Stroke(width = thin),
                )
                drawArc(
                    color = color,
                    startAngle = turned + 45f - SECTOR / 2,
                    sweepAngle = SECTOR,
                    useCenter = false,
                    topLeft = corner,
                    size = box,
                    style = Stroke(width = thin),
                )
            }
        }
    }
}

/** Сколько градусов занимает одна дуга дорожки. */
private const val SECTOR = 45f

/**
 * Во сколько раз уменьшается пластинка в малом виде. Край большой стоит на 0,48 стороны, кольцо
 * спектра начинается на 0,24 — значит доля меньше половины оставляет между ними зазор.
 */
private const val SMALL_SHARE = 0.45f

/**
 * Скорость вращения. 200 градусов в секунду — это 33⅓ оборота в минуту, то есть настоящая скорость
 * долгоиграющей пластинки: полный оборот за 1,8 секунды.
 */
private const val DEGREES_PER_SECOND = 200f
