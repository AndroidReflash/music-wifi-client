package viewModel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dataClasses.TrackInfo
import extensions.toFormattedDuration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.launch
import repository.interfaces.SongsRepository
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.isActive
import kotlinx.coroutines.withContext
import repository.interfaces.NetworkRepository

class SongsScreenViewModel (
    private val scope: CoroutineScope,
    private val songsRepository: SongsRepository,
    private val networkRepository: NetworkRepository,
): ViewModel() {
    val serverIsAlive = networkRepository.serverUp
    val currentPlay : StateFlow<TrackInfo?> = songsRepository.currentPlay
    val songList: StateFlow<List<TrackInfo>> = songsRepository.songList
    val isPlaying: StateFlow<Boolean> = songsRepository.isPlaying

    val equalizerValues : StateFlow<List<Double>> = songsRepository.equalizerArray

    val trackTimeLabel: StateFlow<String> = songsRepository.currentPositionMs.map { ms ->
        ms.toFormattedDuration()
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "00:00")

    val trackSizeLabel: StateFlow<String> = songsRepository.currentTrackLength.map { ms ->
        ms.toFormattedDuration()
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(5000), "00:00")

    val trackProgress: StateFlow<Float> = songsRepository.currentPositionMs
        .combine(songsRepository.currentTrackLength) { position, duration ->
            // Защита от деления на ноль, если данные еще не загружены
            if (duration > 0) position.toFloat() / duration.toFloat() else 0f
        }
        .stateIn(
            scope = viewModelScope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = 0f
        )
    val error : StateFlow<String> = networkRepository.fetchError
    val shuffle: StateFlow<Boolean> = songsRepository.shuffle

    val repeat: StateFlow<Boolean> = songsRepository.repeat

    init {
        scope.launch {
            while (isActive){
                networkRepository.getServerStatus()
                delay(1000)
            }
        }
    }

    fun initSongService(){
        scope.launch {
            withContext(Dispatchers.IO){
                songsRepository.startSongService()
            }
        }
    }
    fun play(
        trackInfo: TrackInfo
    ){
        songsRepository.play(trackInfo)
    }

    fun nextTrack(){
        songsRepository.nextTrack()
    }

    fun previousTrack(){
        songsRepository.prevTrack()
    }

    fun seekTo(percentage: Float){
        songsRepository.seekTo(percentage)
    }

    fun shuffle(){
        songsRepository.shuffle()
    }

    fun repeat(){
        songsRepository.repeat()
    }
}