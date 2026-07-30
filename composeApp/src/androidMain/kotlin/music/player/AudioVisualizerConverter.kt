package music.player

import music.interfaces.AudioVisualizerConverterInterface
import kotlin.math.pow

class AudioVisualizerConverter: AudioVisualizerConverterInterface {
    private val PART_NUMBER = 32
    private val FREQUENCY_EXPONENT = 1.5
    private val RELEVANT_DATA_FACTOR = 0.5

    override fun computeBands(fftArray: List<Double>): List<Double> {
        if (fftArray.isEmpty()) return emptyList()

        val relevantSize = (fftArray.size * RELEVANT_DATA_FACTOR).toInt()
        return List(PART_NUMBER) { i ->
            val startPercent = (i.toDouble() / PART_NUMBER).pow(FREQUENCY_EXPONENT)
            val endPercent = ((i + 1).toDouble() / PART_NUMBER).pow(FREQUENCY_EXPONENT)

            val startIndex = (startPercent * relevantSize).toInt()
            val endIndex = (endPercent * relevantSize).toInt().coerceAtLeast(startIndex + 1)

            var maxVal = 0.0
            for (j in startIndex until endIndex.coerceAtMost(fftArray.size)) {
                if (fftArray[j] > maxVal) maxVal = fftArray[j]
            }
            maxVal
        }
    }
}