package viewModel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dataClasses.TrackInfo
import dispatchers.FolderPickerDispatcher
import extensions.toFormattedDuration
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import repository.interfaces.NetworkRepository
import repository.interfaces.SettingsRepository
import repository.interfaces.SongsRepository

class SongsScreenViewModel (
    private val scope: CoroutineScope,
    private val songsRepository: SongsRepository,
    private val networkRepository: NetworkRepository,
    private val settings: SettingsRepository,
    private val dispatcher: FolderPickerDispatcher
): ViewModel() {
    val isLocal : StateFlow<Boolean> = songsRepository.isLocal
    val currentIp = settings.serverAddress.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = ""
    )
    val folderAddress = settings.folderAddress.stateIn(
        scope = viewModelScope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = ""
    )
    val isServerAvailable = networkRepository.isServerAvailable
    val currentPlay : StateFlow<TrackInfo?> = songsRepository.currentPlay
    val songListOffline: StateFlow<List<TrackInfo>> = songsRepository.songListOffline
    val songListOnline: StateFlow<List<TrackInfo>> = songsRepository.songListOnline
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

    fun setServerIp(url: String){
        scope.launch {
            withContext(Dispatchers.IO){
                settings.setServerAddress(url)
            }
        }
    }

    fun onFolderSelected(uri: String){
        scope.launch {
            withContext(Dispatchers.IO){
                settings.setFolderAddress(uri)
            }
        }
    }

    fun onSelectFolderClicked(){
        scope.launch {
            dispatcher.requestFolder()
        }
    }

    fun isLocal(){
        scope.launch {
            settings.setOnlineToggle(!isLocal.value)
        }
    }

    fun onSelectedManually(trackInfo: TrackInfo){
        songsRepository.onTrackSelectedManually(trackInfo.trackIndex)
    }
}