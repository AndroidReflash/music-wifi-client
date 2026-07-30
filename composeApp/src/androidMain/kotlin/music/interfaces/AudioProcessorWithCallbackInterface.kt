package music.interfaces

import kotlinx.coroutines.flow.StateFlow

interface AudioProcessorWithCallbackInterface {
    val onFftData: StateFlow<List<Double>>
}
