package music.interfaces

interface AudioVisualizerConverterInterface {
    fun computeBands(fftArray: List<Double>): List<Double>
}