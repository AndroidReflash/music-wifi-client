package music.repository

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import androidx.media3.common.util.UnstableApi
import dataClasses.TrackInfo
import keywords.Keywords
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import music.SongService
import music.player.AudioVisualizerConverter
import repository.interfaces.NetworkRepository
import repository.interfaces.OfflineRepository
import repository.interfaces.SettingsRepository
import repository.interfaces.SongsRepository

@androidx.annotation.OptIn(UnstableApi::class)
@OptIn(ExperimentalCoroutinesApi::class)
class SongsRepositoryImpl(
    appContext: Context,
    private val scope: CoroutineScope,
    private val networkRepository: NetworkRepository,
    private val settings: SettingsRepository,
    private val offlineRepository: OfflineRepository
): SongsRepository {

    override val isLocal : StateFlow<Boolean> = settings.isPlayingLocal.stateIn(
        scope = scope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = false
    )
    private val context = appContext.applicationContext
    private var playerService: SongService? = null
    private val _serviceBound = MutableStateFlow(false)
    private val serviceBound: StateFlow<Boolean> = _serviceBound

    private val _songListOffline = MutableStateFlow<List<TrackInfo>>(emptyList())
    override val songListOffline: StateFlow<List<TrackInfo>> = _songListOffline

    private val _songListOnline = MutableStateFlow<List<TrackInfo>>(emptyList())
    override val songListOnline: StateFlow<List<TrackInfo>> = _songListOnline

    var shuffledIndices: List<Int> = emptyList()
    var shufflePointer: Int = 0

    override val currentPlay: StateFlow<TrackInfo?> = serviceBound.flatMapLatest { bound ->
        if (bound && playerService != null) playerService!!.currentPlay else flowOf(null)
    }.stateIn(scope, SharingStarted.Companion.Eagerly, null)

    override val isPlaying: StateFlow<Boolean> = serviceBound.flatMapLatest { bound ->
        if (bound && playerService != null) playerService!!.isPlaying else flowOf(false)
    }.stateIn(scope, SharingStarted.Companion.Eagerly, false)

    private val _equalizerArray = MutableStateFlow<List<Double>>(emptyList())
    override val equalizerArray : StateFlow<List<Double>> = _equalizerArray

    override val currentPositionMs: StateFlow<Long> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.currentPositionMs else MutableStateFlow(0L)
    }.stateIn(scope, SharingStarted.Companion.Eagerly, 0L)


    override val currentTrackLength: StateFlow<Long> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.currentTrackLength else MutableStateFlow(0L)
    }.stateIn(scope, SharingStarted.Companion.Eagerly, 0L)

    override val shuffle: StateFlow<Boolean> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.shuffle else MutableStateFlow(false)
    }.stateIn(scope, SharingStarted.Companion.Eagerly, false)

    override val repeat: StateFlow<Boolean> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.repeat else MutableStateFlow(false)
    }.stateIn(scope, SharingStarted.Companion.Eagerly, false)

    init {
        scope.launch(Dispatchers.Main) {
            launch {
                serviceBound.flatMapLatest { bound ->
                    if (bound && playerService != null) playerService!!.action else flowOf("")
                }.collect { action ->
                    when(action) {
                        Keywords.PlayerPrevItem.value -> {
                            prevTrack()
                            playerService?.setDefaultAction()
                        }
                        Keywords.PlayerNextItem.value -> {
                            nextTrack()
                            playerService?.setDefaultAction()
                        }
                    }
                }
            }

            launch {
                serviceBound.flatMapLatest { bound ->
                    if (bound && playerService != null) {
                        playerService!!.audioProcessor.onFftData
                    } else {
                        flowOf(emptyList<Double>())
                    }
                }.collect { fftData ->
                    _equalizerArray.value = AudioVisualizerConverter.computeBands(fftData)
                }
            }

            launch {
                settings.folderAddress.collect {
                    withContext(Dispatchers.IO){
                        _songListOffline.value = offlineRepository.offlineSongList()
                    }
                }
            }

            networkRepository.observeServerUpdates().collect {
                withContext(Dispatchers.IO) {
                    networkRepository.getServerConfig()
                    _songListOnline.value = networkRepository.fetchPlaylist()
                }
            }
        }
    }

    override fun repeat() {
        if (playerService == null) return
        playerService?.toggleRepeat()
    }

    override fun shuffle() {
        if (playerService == null) return
        playerService?.toggleShuffle()
    }

    override fun play(
        trackInfo: TrackInfo,
    ) {
        if (playerService == null) {
            startSongService()
            scope.launch {
                serviceBound.first { it }
                withContext(Dispatchers.Main){
                    playerService?.togglePlay(trackInfo)
                }
            }
        }else{
            playerService?.togglePlay(trackInfo)
        }
    }

    override fun onTrackSelectedManually(index: Int) {
        if (shuffle.value) {
            val currentSongList = if(isLocal.value) songListOffline.value else songListOnline.value
            val remaining = currentSongList.indices.filter { it != index }.shuffled()
            shuffledIndices = listOf(index) + remaining
            shufflePointer = 0
        }
    }

    override fun nextTrack() {
        if (playerService == null) return
        if (repeat.value && currentPlay.value != null && currentPlay.value?.isLocal == isLocal.value) {
            playerService?.toggleRepeatPlay(currentPlay.value!!)
            return
        }
        val currentIndex = playerService?.currentPlay?.value?.trackIndex ?: return
        val currentSongList = if (isLocal.value) songListOffline.value else songListOnline.value
        if (!currentSongList.isNotEmpty()) return

        if (playerService?.currentPlay?.value?.isLocal == isLocal.value) {
            if (!shuffle.value) {
                val nextTrack = if (currentIndex < currentSongList.size - 1)
                    currentSongList[currentIndex + 1] else currentSongList[0]
                playerService?.togglePlay(nextTrack)
            } else {
                if (shuffledIndices.isEmpty()) {
                    shuffledIndices = currentSongList.indices.shuffled()
                    shufflePointer = 0
                } else {
                    shufflePointer++
                    if (shufflePointer >= shuffledIndices.size) {
                        shuffledIndices = currentSongList.indices.shuffled()
                        shufflePointer = 0
                    }
                }

                val nextTrackIndex = shuffledIndices[shufflePointer]
                playerService?.togglePlay(currentSongList[nextTrackIndex])
            }
        } else {
            playerService?.togglePlay(currentSongList[0])
        }
    }

    override fun prevTrack() {
        if (playerService == null) return
        if (repeat.value && currentPlay.value != null && currentPlay.value?.isLocal == isLocal.value) {
            playerService?.toggleRepeatPlay(currentPlay.value!!)
            return
        }
        val currentIndex = playerService?.currentPlay?.value?.trackIndex ?: return
        val currentSongList = if (isLocal.value) songListOffline.value else songListOnline.value
        if (!currentSongList.isNotEmpty()) return

        if (playerService?.currentPlay?.value?.isLocal == isLocal.value) {
            if (!shuffle.value) {
                val prevTrack = if (currentIndex > 0)
                    currentSongList[currentIndex - 1] else currentSongList[currentSongList.size - 1]
                playerService?.togglePlay(prevTrack)
            } else {
                if (shuffledIndices.isEmpty()) {
                    shuffledIndices = currentSongList.indices.shuffled()
                    shufflePointer = 0
                } else {
                    shufflePointer--
                    if (shufflePointer < 0) {
                        shufflePointer = shuffledIndices.size - 1
                    }
                }

                val prevTrackIndex = shuffledIndices[shufflePointer]
                playerService?.togglePlay(currentSongList[prevTrackIndex])
            }
        } else {
            playerService?.togglePlay(currentSongList[0])
        }
    }

    private fun startSongService(){
        if(!serviceBound.value){
            val intent = Intent(context, SongService::class.java)
            if(playerService == null){
                context.startForegroundService(intent)
            }
            context.bindService(
                intent, serviceConnection, Context.BIND_AUTO_CREATE
            )
        }
    }

    override fun seekTo(percentage: Float) {
        playerService?.seekTo(percentage)
    }

    private val serviceConnection = object : ServiceConnection {
        override fun onServiceConnected(name: ComponentName?, binder: IBinder?) {
            val localBinder = binder as SongService.LocalBinder
            playerService = localBinder.getService()
            _serviceBound.value = true
        }

        override fun onServiceDisconnected(name: ComponentName?) {
            playerService = null
            _serviceBound.value = false
        }
    }
}