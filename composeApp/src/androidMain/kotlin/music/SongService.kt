package music

import android.R
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.Intent
import android.os.Binder
import android.os.IBinder
import android.view.KeyEvent
import androidx.core.app.NotificationCompat
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.util.Log
import androidx.media3.common.util.UnstableApi
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import androidx.media3.session.MediaStyleNotificationHelper
import dataClasses.TrackInfo
import keywords.Keywords
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import music.player.AudioProcessorWithCallback
import music.player.CustomRenderersFactory

@UnstableApi
class SongService: MediaSessionService() {
    private val serviceJob = SupervisorJob()
    private val scope = CoroutineScope(Dispatchers.Main + serviceJob)
    private lateinit var notificationManager: NotificationManager
    private val _mediaSession = MutableStateFlow<MediaSession?>(null)
    private val mediaSession: StateFlow<MediaSession?> = _mediaSession
    private lateinit var exoPlayer: ExoPlayer
    lateinit var audioProcessor: AudioProcessorWithCallback
    private val _currentPlay = MutableStateFlow<TrackInfo?>(null)
    val currentPlay : StateFlow<TrackInfo?> = _currentPlay

    private val _currentPositionMs = MutableStateFlow(0L)
    val currentPositionMs : StateFlow<Long> = _currentPositionMs

    private val _currentTrackLength = MutableStateFlow(0L)
    val currentTrackLength : StateFlow<Long> = _currentTrackLength

    private val _action = MutableStateFlow("")
    val action: StateFlow<String> = _action

    private val _isPlaying = MutableStateFlow(false)
    val isPlaying : StateFlow<Boolean> = _isPlaying

    private val _shuffle = MutableStateFlow(false)
    val shuffle : StateFlow<Boolean> = _shuffle

    private val _repeat = MutableStateFlow(false)
    val repeat : StateFlow<Boolean> = _repeat

    fun toggleShuffle(){
        _shuffle.value = !_shuffle.value
    }

    fun toggleRepeat(){
        _repeat.value = !_repeat.value
    }

    inner class LocalBinder : Binder() {
        // чтобы можно было привязаться к этому же сервису, если система самы вырубит активити
        // а сервис останется в живых
        fun getService(): SongService = this@SongService
    }
    private val binder = LocalBinder()
    override fun onBind(intent: Intent?): IBinder {
        super.onBind(intent)
        return binder
    }

    override fun onCreate() {
        super.onCreate()
        notificationChannel()
        setNotificationManager()
        buildPlayer()
        setNotification()
    }

    override fun onDestroy() {
        super.onDestroy()
        serviceJob.cancel()
        exoPlayer.release()
        _mediaSession.value?.release()
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? {
        return mediaSession.value
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        super.onStartCommand(intent, flags, startId)
        return START_NOT_STICKY
    }

    private val _currentMediaItem = MutableStateFlow<MediaItem?>( null)
    private val currentMediaItem : StateFlow<MediaItem?> = _currentMediaItem

    private fun buildPlayer(){
        audioProcessor = AudioProcessorWithCallback()
        val renderersFactory = CustomRenderersFactory(this, audioProcessor)
        exoPlayer = ExoPlayer.Builder(this, renderersFactory).build()
        exoPlayer.addListener(object : Player.Listener {
            override fun onPlayerError(error: PlaybackException) {
                Log.e("PLAYER-SERVICE", "error = ${error.message}", error)
            }
            override fun onMediaItemTransition(mediaItem: MediaItem?, reason: Int) {
                super.onMediaItemTransition(mediaItem, reason)
                _currentMediaItem.value = mediaItem
            }
            override fun onPlaybackStateChanged(playbackState: Int) {
                super.onPlaybackStateChanged(playbackState)
                when (playbackState) {
                    Player.STATE_IDLE -> {
                        Log.e("StateState", "STATE_IDLE")
                        when(currentMediaItem.value?.mediaId){
                            Keywords.PlayerPrevItem.value -> {
                                _action.value = Keywords.PlayerPrevItem.value
                                Log.e("StateState", "Prev")
                            }
                            Keywords.PlayerNextItem.value -> {
                                _action.value = Keywords.PlayerNextItem.value
                                Log.e("StateState", "Next")
                            }
                        }
                    }
                    Player.STATE_BUFFERING -> {
                        Log.e("StateState", "STATE_BUFFERING")
                    }
                    Player.STATE_READY -> {
                        Log.e("StateState", "STATE_READY")
                        _currentTrackLength.value = exoPlayer.duration
                    }
                    Player.STATE_ENDED -> {
                        Log.e("StateState", "STATE_ENDED")
                    }
                }
            }

            override fun onIsPlayingChanged(isPlaying: Boolean) {
                super.onIsPlayingChanged(isPlaying)
                updateNotification()
                _isPlaying.value = isPlaying
            }
        })
        setMediaSession(exoPlayer)
    }

    fun togglePlay(
        trackInfo: TrackInfo,
    ){
        if(exoPlayer.mediaItemCount == 0 || !trackInfoDiffer(trackInfo)){
            exoPlayer.stop()
            exoPlayer.setMediaItem(MediaItem.fromUri(""), true)
            exoPlayer.setMediaItems(
                mediaItemsList(trackInfo),
                1, C.TIME_UNSET
            )
            exoPlayer.prepare()
            _currentPlay.value = trackInfo
            getProgress()
        }
        if(exoPlayer.isPlaying){
            exoPlayer.pause()
        }else{
            exoPlayer.play()
        }
    }

    fun toggleRepeatPlay(trackInfo: TrackInfo){
        exoPlayer.stop()
        exoPlayer.setMediaItem(MediaItem.fromUri(""), true)
        exoPlayer.setMediaItems(
            mediaItemsList(trackInfo),
            1, C.TIME_UNSET
        )
        exoPlayer.prepare()
        _currentPlay.value = trackInfo
        getProgress()
        exoPlayer.play()
    }

    private fun trackInfoDiffer(trackInfo: TrackInfo): Boolean{
        val first = trackInfo.trackName == currentPlay.value?.trackName
        val second = trackInfo.trackUrl == currentPlay.value?.trackUrl
        val third = trackInfo.trackIndex == currentPlay.value?.trackIndex
        val act = first && second && third
        return act
    }

    private var progressJob : Job? = null
    private fun getProgress() {
        progressJob?.cancel()
        progressJob = null
        progressJob = scope.launch {
            while (isActive) {
                if (exoPlayer.isPlaying) {
                    _currentPositionMs.value = exoPlayer.currentPosition
                }
                delay(1000)
            }
        }
    }

    // медиасессия для контроля происходящего
    private fun setMediaSession(player: ExoPlayer){
        val session = MediaSession.Builder(this@SongService, player)
            .setCallback(object : MediaSession.Callback{
                override fun onMediaButtonEvent(
                    session: MediaSession,
                    controllerInfo: MediaSession.ControllerInfo,
                    intent: Intent
                ): Boolean {
                    if (Intent.ACTION_MEDIA_BUTTON == intent.action) {
                        val keyEvent = intent.getParcelableExtra<KeyEvent>(Intent.EXTRA_KEY_EVENT)
                        keyEvent?.let { event ->
                            if (event.action == KeyEvent.ACTION_DOWN) {
                                when (event.keyCode) {
                                    KeyEvent.KEYCODE_MEDIA_PLAY -> {
                                        if(player.isPlaying){
                                            player.pause()
                                        }else{
                                            player.play()
                                        }
                                    }
                                    KeyEvent.KEYCODE_MEDIA_PAUSE -> {
                                        player.pause()
                                    }
                                    KeyEvent.KEYCODE_MEDIA_NEXT -> {
                                        Log.e("StateState", "NEXT")
                                    }
                                    KeyEvent.KEYCODE_MEDIA_PREVIOUS -> {
                                        Log.e("StateState", "PREVIOUS")
                                    }
                                    KeyEvent.KEYCODE_MEDIA_STOP -> {
                                        player.stop()
                                    }
                                }
                            }
                        }
                        return true
                    }
                    return super.onMediaButtonEvent(session, controllerInfo, intent)
                }
            })
            .setId(Keywords.PlayerId.value)
        _mediaSession.value?.release()
        _mediaSession.value = session.build()
    }

    private fun setNotification(){
        startForeground(128, buildNotification())
    }

    private fun setNotificationManager(){
        notificationManager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
    }

    private fun updateNotification() {
        notificationManager.notify(128, buildNotification())
    }
    private fun buildNotification(): Notification{
        return NotificationCompat.Builder(this@SongService, "128")
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .setSmallIcon(R.drawable.ic_media_play)
            .setCategory(Notification.CATEGORY_TRANSPORT)
            .setStyle(MediaStyleNotificationHelper.MediaStyle(mediaSession.value!!))
            .setOngoing(true).build()
    }

    // сервис не будет работать, если не сделать канал для уведомлений
    private fun notificationChannel(){
        val channel = NotificationChannel(
            "128",
            "Фоновый процесс для плееров",
            NotificationManager.IMPORTANCE_LOW
        )
        val manager = getSystemService(NotificationManager::class.java)
        manager.createNotificationChannel(channel)
    }

    private fun mediaItemsList(trackInfo: TrackInfo): List<MediaItem>{
        val mediaItem = MediaItem.Builder()
            .setUri(trackInfo.trackUrl)
            .setMimeType(MimeTypes.AUDIO_MPEG)
            .setMediaMetadata(
                MediaMetadata.Builder()
                    .setTitle(trackInfo.trackName)
                    .build()
            )
            .build()
        val nextPlaceholder = MediaItem.Builder()
            .setUri(Keywords.PlayerNextURL.value)
            .setMediaId(Keywords.PlayerNextItem.value)
            .build()
        val prevPlaceholder = MediaItem.Builder()
            .setUri(Keywords.PlayerPrevURL.value)
            .setMediaId(Keywords.PlayerPrevItem.value)
            .build()

        return listOf(prevPlaceholder, mediaItem, nextPlaceholder)
    }

    fun setDefaultAction(){
        _action.value = Keywords.DefaultAction.value
    }

    fun seekTo(positionPercent: Float) {
        val totalDuration = exoPlayer.duration
        if (totalDuration > 0 ) {
            val seekPosition = (totalDuration * positionPercent).toLong()
            exoPlayer.seekTo(seekPosition)
        }
    }
}