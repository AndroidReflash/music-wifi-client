package ui.songScreenElements

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import kotlin.math.pow

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
    amplitudes: List<Double>,
    modifier: Modifier = Modifier,
    baseColor: Color
) {

    Canvas(modifier = modifier.size(width = 400.dp, height = 300.dp)) {
        val canvasWidth = size.width
        val canvasHeight = size.height

        val totalBands = 40
        val halfBands = totalBands / 2

        val gap = 4.dp.toPx()
        val bandWidth = (canvasWidth - (gap * (totalBands - 1))) / totalBands

        if (amplitudes.isNotEmpty()) {
            val step = amplitudes.size / halfBands
            val halfAmplitudes = List(halfBands) { index ->
                val sourceIndex = (index * step).coerceIn(0, amplitudes.lastIndex)
                amplitudes[sourceIndex]
            }
            val mirroredAmplitudes = halfAmplitudes.reversed() + halfAmplitudes

            mirroredAmplitudes.forEachIndexed { index, amplitude ->
                val finalAmplitude = amplitude.pow(0.5).toFloat()

                val minHeight = 6.dp.toPx()
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
}
