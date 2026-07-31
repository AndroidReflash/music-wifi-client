package platform

import android.content.ContentResolver
import android.content.Context
import android.graphics.Bitmap
import android.net.Uri
import android.os.Looper
import androidx.annotation.DrawableRes
import androidx.core.content.ContextCompat
import androidx.core.graphics.drawable.toBitmap
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.Player
import androidx.media3.common.SimpleBasePlayer
import androidx.media3.common.util.UnstableApi
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture
import dataClasses.TrackInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch
import repository.interfaces.SongsRepository
import kotlinx.coroutines.runBlocking
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.getDrawableResourceBytes
import org.jetbrains.compose.resources.getSystemResourceEnvironment
import wifi.music.list.R
import java.io.ByteArrayOutputStream

/**
 * Плеер, каким его видит медиасессия. Сам он ничего не играет: звук делает Rust, а здесь
 * только пересказ состояния из [SongsRepository] на языке media3 и обратный перевод
 * нажатий — из уведомления, с экрана блокировки и с кнопок гарнитуры.
 *
 * Своего интерфейса у него нет и не нужно: контракт задаёт `androidx.media3.common.Player`,
 * а `SimpleBasePlayer` — это его готовая основа для чужих движков: наследник отдаёт
 * состояние целиком и разбирает приказы, а разницу между прежним и новым состоянием
 * media3 считает сам.
 *
 * Всё, что здесь есть, живёт на одном потоке — том, чей [Looper] передан при создании.
 * Оттуда же зовётся [invalidateState], иначе media3 сочтёт состояние испорченным.
 */
@UnstableApi
class RustMediaPlayer(
    looper: Looper,
    scope: CoroutineScope,
    private val repository: SongsRepository,
    private val context: Context
) : SimpleBasePlayer(looper) {
    /**
     * Что нам позволено. Списка дорожек с произвольной перестановкой здесь нет: очередью
     * распоряжается Rust, и media3 её только показывает.
     */
    private val commands = Player.Commands.Builder()
        .addAll(
            Player.COMMAND_PLAY_PAUSE,
            Player.COMMAND_PREPARE,
            Player.COMMAND_STOP,
            Player.COMMAND_SEEK_TO_NEXT,
            Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM,
            Player.COMMAND_SEEK_TO_PREVIOUS,
            Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM,
            Player.COMMAND_SEEK_IN_CURRENT_MEDIA_ITEM,
            Player.COMMAND_SEEK_TO_MEDIA_ITEM,
            Player.COMMAND_GET_CURRENT_MEDIA_ITEM,
            Player.COMMAND_GET_TIMELINE,
            Player.COMMAND_GET_METADATA,
            Player.COMMAND_SET_SHUFFLE_MODE,
            Player.COMMAND_SET_REPEAT_MODE,
        )
        .build()

    init {
        // Позиции в этом списке нет намеренно: её media3 спрашивает сам, через
        // поставщика в состоянии, и пересобирать состояние пять раз в секунду незачем.
        //
        // Диспетчер главного потока, а не его `immediate`: все потоки состояния уже
        // имеют значение и выдали бы его прямо здесь, то есть из конструктора, а media3
        // ждёт первого обновления после создания.
        scope.launch(Dispatchers.Main) {
            combine(
                repository.isPlaying,
                repository.currentPlay,
                repository.currentTrackLength,
                repository.shuffle,
                repository.repeat,
            ) { _, _, _, _, _ -> Unit }.collect { invalidateState() }
        }

        scope.launch(Dispatchers.Main) {
            repository.playlist.collect { invalidateState() }
        }

        // Обложка приходит позже дорожки: её читают из файла, и к первому пересчёту состояния её
        // ещё нет. Без этого уведомление осталось бы без картинки до следующей смены дорожки.
        scope.launch(Dispatchers.Main) {
            repository.cover.collect { invalidateState() }
        }
    }

    override fun getState(): State {
        val tracks = repository.playlist.value
        val current = repository.currentPlay.value
        val index = tracks.indexOf(current)
        return State.Builder()
            .setAvailableCommands(commands)
            .setPlaybackState(if (tracks.isEmpty()) Player.STATE_IDLE else Player.STATE_READY)
            .setPlayWhenReady(
                repository.isPlaying.value,
                Player.PLAY_WHEN_READY_CHANGE_REASON_USER_REQUEST,
            )
            .setPlaylist(tracks.map { track -> itemOf(track, track == current) })
            .setCurrentMediaItemIndex(if (index >= 0) index else 0)
            .setContentPositionMs(PositionSupplier { repository.currentPositionMs.value })
            .setShuffleModeEnabled(repository.shuffle.value)
            .setRepeatMode(
                if (repository.repeat.value) Player.REPEAT_MODE_ONE else Player.REPEAT_MODE_OFF,
            )
            .build()
    }

    override fun handleSetPlayWhenReady(playWhenReady: Boolean): ListenableFuture<*> {
        repository.setPlaying(playWhenReady)
        return Futures.immediateVoidFuture()
    }

    override fun handlePrepare(): ListenableFuture<*> = Futures.immediateVoidFuture()

    /** Остановки у репозитория нет: ближайшее к ней — перестать играть. */
    override fun handleStop(): ListenableFuture<*> {
        repository.setPlaying(false)
        return Futures.immediateVoidFuture()
    }

    override fun handleSeek(
        mediaItemIndex: Int,
        positionMs: Long,
        seekCommand: Int,
    ): ListenableFuture<*> {
        when (seekCommand) {
            Player.COMMAND_SEEK_TO_NEXT,
            Player.COMMAND_SEEK_TO_NEXT_MEDIA_ITEM,
            -> repository.nextTrack()

            Player.COMMAND_SEEK_TO_PREVIOUS,
            Player.COMMAND_SEEK_TO_PREVIOUS_MEDIA_ITEM,
            -> repository.prevTrack()

            else -> seekWithin(mediaItemIndex, positionMs)
        }
        return Futures.immediateVoidFuture()
    }

    override fun handleSetShuffleModeEnabled(shuffleModeEnabled: Boolean): ListenableFuture<*> {
        if (repository.shuffle.value != shuffleModeEnabled) repository.shuffle()
        return Futures.immediateVoidFuture()
    }

    /** Повтор у нас двоичный: одна дорожка по кругу или ничего. */
    override fun handleSetRepeatMode(repeatMode: Int): ListenableFuture<*> {
        val one = repeatMode == Player.REPEAT_MODE_ONE
        if (repository.repeat.value != one) repository.repeat()
        return Futures.immediateVoidFuture()
    }

    /**
     * Перемотка по ползунку уведомления или переход на дорожку списка. Репозиторий
     * принимает долю дорожки, а media3 даёт миллисекунды, — отсюда деление.
     */
    private fun seekWithin(mediaItemIndex: Int, positionMs: Long) {
        val track = repository.playlist.value.getOrNull(mediaItemIndex) ?: return
        if (track != repository.currentPlay.value) {
            repository.play(track)
            return
        }
        val length = repository.currentTrackLength.value
        if (length <= 0L || positionMs == C.TIME_UNSET) return
        repository.seekTo(positionMs.toFloat() / length.toFloat())
    }

    /**
     * Дорожка для media3. Длительность известна только у играющей — её знает декодер, а
     * до открытия остальных её не знает никто.
     */
    private fun itemOf(track: TrackInfo, current: Boolean): MediaItemData {
        val lengthMs = repository.currentTrackLength.value
        val durationUs = if (current && lengthMs > 0L) lengthMs * 1000L else C.TIME_UNSET
        // Картинка нужна только играющей дорожке: уведомление показывает одну, а держать байты
        // всего списка — это мегабайты на каждую песню.
        val artwork = if (current) repository.cover.value else null
        val mediaItem = MediaItem.Builder()
            .setUri(track.trackUrl)
            .setMediaMetadata(
                MediaMetadata.Builder()
                    .setTitle(track.trackName)
                    .setArtworkData(artwork, MediaMetadata.PICTURE_TYPE_FRONT_COVER)
                    .build(),
            )
            .build()
        val defaultArtworkUri = Uri.Builder()
            .scheme(ContentResolver.SCHEME_ANDROID_RESOURCE)
            .authority(context.packageName)
            .appendPath(R.drawable.image.toString())
            .build()
        val mediaItemDef = MediaItem.Builder()
            .setUri(track.trackUrl)
            .setMediaMetadata(
                MediaMetadata.Builder()
                    .setTitle(track.trackName)
                    .setArtworkUri(defaultArtworkUri)
                    .build(),
            )
            .build()
        return MediaItemData.Builder("${track.trackIndex}:${track.trackUrl}")
            .setMediaItem(if (artwork == null) mediaItemDef else mediaItem)
            .setDurationUs(durationUs)
            .build()
    }
}
