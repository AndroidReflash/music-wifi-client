package music

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import androidx.annotation.OptIn
import androidx.media3.common.util.UnstableApi
import dataClasses.TrackInfo
import keywords.Keywords
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import music.player.AudioVisualizerConverter
import repository.interfaces.NetworkRepository
import repository.interfaces.SongsRepository

@OptIn(UnstableApi::class)
@kotlin.OptIn(ExperimentalCoroutinesApi::class)
class SongsRepositoryImpl(
    appContext: Context,
    scope: CoroutineScope,
    private val networkRepository: NetworkRepository,
): SongsRepository {
    private val context = appContext.applicationContext
    private var playerService: SongService? = null
    private val _serviceBound = MutableStateFlow(false)
    private val serviceBound: StateFlow<Boolean> = _serviceBound
    private val _songList = MutableStateFlow<List<TrackInfo>>(emptyList())
    override val songList: StateFlow<List<TrackInfo>> = _songList


    override val currentPlay: StateFlow<TrackInfo?> = serviceBound.flatMapLatest { bound ->
        if (bound && playerService != null) playerService!!.currentPlay else flowOf(null)
    }.stateIn(scope, SharingStarted.Eagerly, null)

    override val isPlaying: StateFlow<Boolean> = serviceBound.flatMapLatest { bound ->
        if (bound && playerService != null) playerService!!.isPlaying else flowOf(false)
    }.stateIn(scope, SharingStarted.Eagerly, false)

    private val _equalizerArray = MutableStateFlow<List<Double>>(emptyList())
    override val equalizerArray : StateFlow<List<Double>> = _equalizerArray

    override val currentPositionMs: StateFlow<Long> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.currentPositionMs else MutableStateFlow(0L)
    }.stateIn(scope, SharingStarted.Eagerly, 0L)


    override val currentTrackLength: StateFlow<Long> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.currentTrackLength else MutableStateFlow(0L)
    }.stateIn(scope, SharingStarted.Eagerly, 0L)

    override val shuffle: StateFlow<Boolean> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.shuffle else MutableStateFlow(false)
    }.stateIn(scope, SharingStarted.Eagerly, false)

    override val repeat: StateFlow<Boolean> = serviceBound.flatMapLatest { bound ->
        if (bound) playerService!!.repeat else MutableStateFlow(false)
    }.stateIn(scope, SharingStarted.Eagerly, false)

    init {
        scope.launch (Dispatchers.IO) {
            // тут не менять - единственный хардкод-эндпоинт
            networkRepository.getServerConfig()

            networkRepository.serverUp.collect {
                _songList.value = networkRepository.fetchPlaylist()
            }
        }
        scope.launch {
            serviceBound.flatMapLatest { bound ->
                if (bound && playerService != null) playerService!!.action else flowOf("")
            }.collect { action ->
                withContext(Dispatchers.Main){
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
        }
        scope.launch {
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
        trackInfo: TrackInfo
    ) {
        if (playerService == null) return
        playerService?.togglePlay(trackInfo)
    }

    override fun nextTrack() {
        if (playerService == null) return
        if(repeat.value && currentPlay.value != null){
            playerService?.toggleRepeatPlay(currentPlay.value!!)
            return
        }
        val currentIndex = playerService?.currentPlay?.value?.trackIndex ?: return
        if(!shuffle.value){
            val nextTrack = if(currentIndex < songList.value.size-1)
                songList.value[currentIndex+1] else songList.value[0]
            playerService?.togglePlay(nextTrack)
        }else {
            val possibleIndices = songList.value.indices
            val filteredIndices = possibleIndices.filter { it != currentIndex }
            val randomIndex = if (filteredIndices.isNotEmpty()) {
                filteredIndices.random()
            } else {
                currentIndex
            }
            playerService?.togglePlay(songList.value[randomIndex])
        }
    }

    override fun prevTrack() {
        if (playerService == null) return
        if(repeat.value && currentPlay.value != null){
            playerService?.toggleRepeatPlay(currentPlay.value!!)
            return
        }
        val currentIndex = playerService?.currentPlay?.value?.trackIndex ?: return
        if(!shuffle.value){
            val prevTrack = if(currentIndex > 0)
                songList.value[currentIndex-1] else songList.value[songList.value.size-1]
            playerService?.togglePlay(prevTrack)
        }else {
            val possibleIndices = songList.value.indices
            val filteredIndices = possibleIndices.filter { it != currentIndex }
            val randomIndex = if (filteredIndices.isNotEmpty()) {
                filteredIndices.random()
            } else {
                currentIndex
            }
            playerService?.togglePlay(songList.value[randomIndex])
        }
    }

    override fun startSongService(){
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