package viewModel

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ImageBitmap
import dataClasses.Hotkey
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dataClasses.TrackInfo
import dispatchers.FolderPickerDispatcher
import enums.HotkeyAction
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
    val interfaceColor: StateFlow<Color> = settings.interfaceColor
    val showVisualizer: StateFlow<Boolean> = settings.showVisualizer
    val showCover: StateFlow<Boolean> = settings.showCover
    val outputDevice: StateFlow<String> = settings.outputDevice
    val outputDevices: StateFlow<List<String>> = songsRepository.outputDevices
    val volume: StateFlow<Float> = settings.volume
    val equalizerBands: List<Float> = songsRepository.equalizerBands
    val equalizerMaxDb: Float = songsRepository.equalizerMaxDb
    val equalizerGains: StateFlow<List<Float>> = songsRepository.equalizerGains
    val hotkeys: StateFlow<Map<HotkeyAction, Hotkey>> = settings.hotkeys
    val hotkeysGlobal: StateFlow<Boolean> = settings.hotkeysGlobal
    val autoplay: StateFlow<Boolean> = settings.autoplay
    val visualizerRing: StateFlow<Boolean> = settings.visualizerRing
    val coverSmall: StateFlow<Boolean> = settings.coverSmall
    val showCoverImage: StateFlow<Boolean> = settings.showCoverImage
    val coverImage: StateFlow<ImageBitmap?> = songsRepository.coverImage
    val panelDivider: StateFlow<Boolean> = settings.panelDivider
    val panelTransparent: StateFlow<Boolean> = settings.panelTransparent
    val isServerAvailable = networkRepository.isServerAvailable
    val currentPlay : StateFlow<TrackInfo?> = songsRepository.currentPlay
    val playlist: StateFlow<List<TrackInfo>> = songsRepository.playlist
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
    val playbackError : StateFlow<String> = songsRepository.playbackError
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

    fun setThemeColor(color: Color) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setThemeColor(color)
            }
        }
    }

    fun setShowVisualizer(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setShowVisualizer(value)
            }
        }
    }

    fun setShowCover(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setShowCover(value)
            }
        }
    }

    fun setOutputDevice(name: String) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setOutputDevice(name)
            }
        }
    }

    fun refreshOutputDevices() = songsRepository.refreshOutputDevices()

    fun setHotkey(action: HotkeyAction, hotkey: Hotkey?) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setHotkey(action, hotkey)
            }
        }
    }

    fun setHotkeysGlobal(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setHotkeysGlobal(value)
            }
        }
    }

    fun setAutoplay(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setAutoplay(value)
            }
        }
    }

    fun stopAutoplay() = songsRepository.stopAutoplay()

    fun setVolume(volume: Float) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setVolume(volume)
            }
        }
    }

    fun setEqualizerBand(band: Int, gainDb: Float) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setEqualizerBand(band, gainDb)
            }
        }
    }

    fun resetEqualizer() {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.resetEqualizer()
            }
        }
    }

    fun setVisualizerRing(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setVisualizerRing(value)
            }
        }
    }

    fun setCoverSmall(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setCoverSmall(value)
            }
        }
    }

    fun setShowCoverImage(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setShowCoverImage(value)
            }
        }
    }

    fun setPanelDivider(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setPanelDivider(value)
            }
        }
    }

    fun setPanelTransparent(value: Boolean) {
        scope.launch {
            withContext(Dispatchers.IO) {
                settings.setPanelTransparent(value)
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
