package music.player

import androidx.media3.common.audio.AudioProcessor
import androidx.media3.common.audio.AudioProcessor.EMPTY_BUFFER
import androidx.media3.common.util.UnstableApi
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import java.nio.ByteBuffer
import org.apache.commons.math3.transform.DftNormalization
import org.apache.commons.math3.transform.FastFourierTransformer
import org.apache.commons.math3.transform.TransformType
import java.nio.ByteOrder

private const val fftSize = 1024 // => вернёт 512 амплитуд
//создал аудиопроцессор, чтобы подменить им стандартный процессор exoPlayer, чтобы можно было звук забирать
@UnstableApi
class AudioProcessorWithCallback : AudioProcessor {

    private val _onFftData = MutableStateFlow<List<Double>>(emptyList())
    val onFftData: StateFlow<List<Double>> = _onFftData

    private var outputBuffer: ByteBuffer = AudioProcessor.EMPTY_BUFFER
    private var pendingOutputBuffer: ByteBuffer = AudioProcessor.EMPTY_BUFFER
    private var inputEnded = false

    private val fftAnalyzer = FftAnalyzer()
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)
    // Ограничим емкость, чтобы не забивать память, если FFT не успевает
    private val fftChannel = Channel<DoubleArray>(capacity = 5)

    init {
        scope.launch {
            for (floatArray in fftChannel) {
                val fftAmplitudes = fftAnalyzer.analyze(floatArray)
                _onFftData.value = fftAmplitudes.toList()
            }
        }
    }

    override fun configure(inputAudioFormat: AudioProcessor.AudioFormat): AudioProcessor.AudioFormat {
        // Мы не меняем формат (PCM 16-bit остается PCM 16-bit)
        return inputAudioFormat
    }

    override fun isActive(): Boolean = true

    override fun queueInput(inputBuffer: ByteBuffer) {
        if (!inputBuffer.hasRemaining()) return

        // 1. Копируем данные для FFT, не мешая основному потоку
        val count = inputBuffer.remaining()
        val bytes = ByteArray(count)
        val duplicate = inputBuffer.duplicate()
        duplicate.get(bytes)

        // Отправляем на обработку (превратим в Double в фоновом потоке!)
        scope.launch {
            val doubles = DoubleArray(bytes.size / 2)
            val buffer = ByteBuffer.wrap(bytes).order(ByteOrder.LITTLE_ENDIAN)
            var i = 0
            while (buffer.remaining() >= 2) {
                doubles[i++] = buffer.short.toDouble() / Short.MAX_VALUE
            }
            fftChannel.trySend(doubles)
        }

        // 2. ОЧЕНЬ ВАЖНО: отдаем данные дальше
        // Мы должны переложить данные из input в output,
        // чтобы ExoPlayer считал, что мы их обработали
        if (pendingOutputBuffer.capacity() < count) {
            pendingOutputBuffer = ByteBuffer.allocateDirect(count).order(ByteOrder.nativeOrder())
        } else {
            pendingOutputBuffer.clear()
        }

        pendingOutputBuffer.put(inputBuffer) // Теперь inputBuffer.position == limit
        pendingOutputBuffer.flip()
        outputBuffer = pendingOutputBuffer
    }

    override fun getOutput(): ByteBuffer {
        val out = outputBuffer
        outputBuffer = AudioProcessor.EMPTY_BUFFER
        return out
    }

    override fun flush() {
        outputBuffer = AudioProcessor.EMPTY_BUFFER
        pendingOutputBuffer.clear()
        inputEnded = false
    }

    override fun reset() {
        flush()
        pendingOutputBuffer = AudioProcessor.EMPTY_BUFFER
    }

    override fun isEnded(): Boolean = inputEnded && outputBuffer === AudioProcessor.EMPTY_BUFFER

    override fun queueEndOfStream() {
        inputEnded = true
    }
}

private class FftAnalyzer {

    private val transformer = FastFourierTransformer(DftNormalization.UNITARY)

    fun analyze(samples: DoubleArray): DoubleArray {
        val paddedSamples = samples.copyOf(fftSize)

        val result = transformer.transform(paddedSamples, TransformType.FORWARD)

        // Сразу создаём массив нужного размера и заполняем
        val amplitudes = DoubleArray(result.size / 2)  // Только первая половина спектра
        for (i in amplitudes.indices) {
            amplitudes[i] = result[i].abs()
        }
        return amplitudes
    }
}