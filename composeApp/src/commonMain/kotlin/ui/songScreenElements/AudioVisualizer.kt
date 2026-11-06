package ui.songScreenElements

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin

@Composable
fun AudioVisualizer(
    amplitudes: List<Double>,
    modifier: Modifier = Modifier,
    baseColor: Color = Color.Red
) {
    Canvas(modifier = modifier.fillMaxSize()) {
        val canvasWidth = size.width
        val canvasHeight = size.height

        val mirroredAmplitudes = amplitudes.reversed() + amplitudes
        val totalBands = mirroredAmplitudes.size

        val gap = 2.dp.toPx()
        val bandWidth = (canvasWidth - (gap * (totalBands - 1))) / totalBands

        mirroredAmplitudes.forEachIndexed { index, amplitude ->
            val minHeight = 4.dp.toPx()

            val left = if (index > 0) mirroredAmplitudes[index - 1] else amplitude
            val right = if (index < mirroredAmplitudes.size - 1) mirroredAmplitudes[index + 1] else amplitude
            val avgAmplitude = (left + amplitude + right) / 3.0

            val finalAmplitude = avgAmplitude.pow(0.5).toFloat()

            val animatedHeight = (finalAmplitude * canvasHeight).coerceIn(minHeight, canvasHeight)

            val xOffset = index * (bandWidth + gap)
            val yOffset = (canvasHeight - animatedHeight) / 2

            drawRoundRect(
                color = baseColor,
                topLeft = Offset(xOffset, yOffset),
                size = Size(bandWidth, animatedHeight),
                style = Stroke(width = 6f),
                cornerRadius = CornerRadius(bandWidth / 2, bandWidth / 2)
            )
        }
    }
}

@Composable
fun AudioVisualizerNoBounds(
    amplitudes: List<Double>,
    modifier: Modifier = Modifier,
    baseColor: Color = Color.Red
) {
    Canvas(modifier = modifier) {
        val canvasWidth = size.width
        val canvasHeight = size.height

        val mirroredAmplitudes = amplitudes.reversed() + amplitudes
        val totalBands = mirroredAmplitudes.size

        val gap = 2.dp.toPx()
        val bandWidth = (canvasWidth - (gap * (totalBands - 1))) / totalBands

        mirroredAmplitudes.forEachIndexed { index, amplitude ->
            val minHeight = 4.dp.toPx()
            val animatedHeight = (amplitude.toFloat() * canvasHeight)
                .coerceAtLeast(minHeight)

            val xOffset = index * (bandWidth + gap)
            val yOffset = (canvasHeight - animatedHeight) / 2

            drawRoundRect(
                color = baseColor,
                topLeft = Offset(xOffset, yOffset),
                size = Size(bandWidth, animatedHeight),
                style = Stroke(width = 6f),
                cornerRadius = CornerRadius(bandWidth / 2, bandWidth / 2)
            )
        }
    }
}

@Composable
fun AudioVisualizerHeight(
    equalizer: List<Double>,
    modifier: Modifier = Modifier,
    baseColor: Color
) {
    val count = (equalizer.size * KEEP_PERCENT).toInt().coerceAtLeast(1)
    val amplitudes = equalizer.dropLast(count)

    // Сохраняем предыдущие значения для плавного спада (decay / lerp)
    var previousAmplitudes = remember { mutableStateOf(listOf<Float>()) }

    Canvas(modifier = modifier.size(width = 500.dp, height = 400.dp)) {
        val canvasWidth = size.width
        val canvasHeight = size.height

        val totalBands = TOTAL_ELEMENTS
        val halfBands = totalBands / 2

        val gap = 1.5.dp.toPx()
        val totalGapsWidth = gap * (totalBands - 1)
        val bandWidth = ((canvasWidth - totalGapsWidth) / totalBands).coerceAtLeast(1f)

        if (amplitudes.isEmpty()) {
            return@Canvas
        }

        // 1. Логарифмическое распределение для половины колонок
        val targetHalfAmplitudes = List(halfBands) { index ->
            if (amplitudes.size == 1) {
                amplitudes[0].toFloat()
            } else {
                val normalizedIndex = index.toFloat() / (halfBands.coerceAtLeast(2) - 1)

                // Степень < 1.0 (например, 0.45) дает мягкое логарифмическое растяжение
                val curvedIndex = normalizedIndex.pow(0.45f)

                val exactIndex = curvedIndex * amplitudes.lastIndex
                val lowerIndex = exactIndex.toInt().coerceIn(0, amplitudes.lastIndex)
                val upperIndex = (lowerIndex + 1).coerceIn(0, amplitudes.lastIndex)
                val fraction = exactIndex - lowerIndex

                val val1 = amplitudes[lowerIndex]
                val val2 = amplitudes[upperIndex]
                (val1 + (val2 - val1) * fraction).toFloat()
            }
        }

        // Зеркалируем для симметричного эквалайзера (из центра в стороны)
        val targetAmplitudes = targetHalfAmplitudes.reversed() + targetHalfAmplitudes

        // 2. Сглаживание (Lerp + Decay) между кадрами
        val smoothedAmplitudes = if (previousAmplitudes.value.size != targetAmplitudes.size) {
            targetAmplitudes
        } else {
            targetAmplitudes.zip(previousAmplitudes.value) { target, prev ->
                if (target > prev) {
                    prev + (target - prev) * 0.7f // Быстрый взлет на ударе
                } else {
                    prev + (target - prev) * 0.18f // Плавное гравитационное падение
                }
            }
        }
        previousAmplitudes.value = smoothedAmplitudes

        smoothedAmplitudes.forEachIndexed { index, amplitude ->
            val freqWeight = START_WEIGHT

            val finalAmplitude = (amplitude.coerceAtLeast(0.0f).pow(WEIGHT_COEFFICIENT) * SCALE_FACTOR) * freqWeight

            val minHeight = 1.dp.toPx()
            val animatedHeight = (finalAmplitude * canvasHeight).coerceIn(minHeight, canvasHeight)

            val xOffset = index * (bandWidth + gap)
            val yOffset = (canvasHeight - animatedHeight) / 2

            drawRoundRect(
                color = baseColor,
                topLeft = Offset(xOffset, yOffset),
                size = Size(bandWidth, animatedHeight),
                cornerRadius = CornerRadius(bandWidth / 2, bandWidth / 2)
            )
        }
    }
}

/**
 * Круговой спектр: те же полосы, но лучами по кольцу.
 *
 * Полосы, их выборка и кривая — те же, что у [AudioVisualizerHeight]: сорок штук, половина берётся
 * из спектра через шаг и зеркалится, громкость под корнем. Меняется только раскладка, поэтому и
 * движение на глаз остаётся прежним.
 *
 * Кольцо считается от меньшей стороны, поэтому рисунок круглый в любой рамке, а его внутренний и
 * внешний радиусы подобраны под нарисованную пластинку: полосы начинаются за кружком этикетки и не
 * доходят до края. Если пластинку спрятать, кольцо просто висит само по себе.
 *
 * Половина спектра кладётся по кругу дважды и в одном и том же порядке. Отражение слева направо
 * вместе с отражением сверху вниз — это в сумме поворот на половину оборота, поэтому рисунок
 * получается симметричным относительно центра, а не относительно вертикали: один и тот же луч
 * стоит и на углу, и напротив него. На круге это читается как вращение, а зеркало относительно
 * вертикали читалось бы как бабочка.
 */
@Composable
fun AudioVisualizerRadial(
    equalizer: List<Double>,
    modifier: Modifier = Modifier,
    baseColor: Color
) {
    // Отсекаем верхний мертвый хвост спектра (выше ~8-10 кГц, где почти ничего нет)
    val count = (equalizer.size * KEEP_PERCENT).toInt().coerceAtLeast(1)
    val amplitudes = equalizer.dropLast(count)

    var previousAmplitudes = remember { mutableStateOf(listOf<Float>()) }

    Canvas(modifier = modifier) {
        if (amplitudes.isEmpty()) {
            return@Canvas
        }

        val side = min(size.width, size.height)
        val center = Offset(size.width / 2, size.height / 2)
        val totalBands = TOTAL_ELEMENTS

        // Логарифмическое распределение: сгущаем выборку в НАЧАЛЕ массива (где басы)
        val targetAmplitudes = List(totalBands) { index ->
            if (amplitudes.size == 1) {
                amplitudes[0].toFloat()
            } else {
                val normalizedIndex = index.toFloat() / (totalBands - 1)

                // Степень < 1.0 растягивает начало массива (басы) для детального отображения
                val curvedIndex = normalizedIndex.pow(0.45f)

                val exactIndex = curvedIndex * amplitudes.lastIndex
                val lowerIndex = exactIndex.toInt().coerceIn(0, amplitudes.lastIndex)
                val upperIndex = (lowerIndex + 1).coerceIn(0, amplitudes.lastIndex)
                val fraction = exactIndex - lowerIndex

                val val1 = amplitudes[lowerIndex]
                val val2 = amplitudes[upperIndex]
                (val1 + (val2 - val1) * fraction).toFloat()
            }
        }

        // Плавный сглаживающий спад (Lerp + Decay)
        val smoothedAmplitudes = if (previousAmplitudes.value.size != targetAmplitudes.size) {
            targetAmplitudes
        } else {
            targetAmplitudes.zip(previousAmplitudes.value) { target, prev ->
                if (target > prev) {
                    prev + (target - prev) * 0.7f
                } else {
                    prev + (target - prev) * 0.18f
                }
            }
        }
        previousAmplitudes.value = smoothedAmplitudes

        val innerRadius = side * INNER_SHARE
        val outerRadius = side * OUTER_SHARE
        val minLength = 1.dp.toPx()
        val maxAvailableLength = outerRadius - innerRadius

        val spacing = (2f * PI.toFloat() * innerRadius) / totalBands
        val thickness = (spacing * BAND_SHARE).coerceAtLeast(1.dp.toPx())
        val degreesPerBand = 360f / totalBands

        smoothedAmplitudes.forEachIndexed { index, amplitude ->
            val normalizedIndex = index.toFloat() / (totalBands - 1)
            val freqWeight = START_WEIGHT + (END_WEIGHT - START_WEIGHT) * normalizedIndex.pow(1.5f)

            val loudness = (amplitude.coerceAtLeast(0.0f).pow(WEIGHT_COEFFICIENT) * SCALE_FACTOR) * freqWeight
            val length = (loudness * maxAvailableLength).coerceIn(minLength, maxAvailableLength)

            val radians = ((index + 0.5f) * degreesPerBand - 90f) * PI.toFloat() / 180f
            val direction = Offset(cos(radians), sin(radians))

            drawLine(
                color = baseColor,
                start = center + direction * innerRadius,
                end = center + direction * (innerRadius + length),
                strokeWidth = thickness,
                cap = StrokeCap.Round,
            )
        }
    }
}

private const val INNER_SHARE = 0.24f

private const val OUTER_SHARE = 0.5f

private const val BAND_SHARE = 0.5f
private const val TOTAL_ELEMENTS = 180
private const val KEEP_PERCENT = 0.3f
private const val SCALE_FACTOR = 1f
private const val WEIGHT_COEFFICIENT = 0.4f
private const val START_WEIGHT = 1f
private const val END_WEIGHT = 1.3f
