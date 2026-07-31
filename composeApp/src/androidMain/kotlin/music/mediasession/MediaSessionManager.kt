package music.mediasession

import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.KeyEvent
import androidx.media3.common.Player
import androidx.media3.common.util.UnstableApi
import androidx.media3.session.CommandButton
import androidx.media3.session.MediaSession
import androidx.media3.session.SessionCommand
import androidx.media3.session.SessionResult
import com.google.common.util.concurrent.Futures
import com.google.common.util.concurrent.ListenableFuture
import keywords.Keywords
import music.SongService
import repository.interfaces.SongsRepository

/**
 * Медиасессия: уведомление, экран блокировки и кнопки гарнитуры.
 *
 * Плеер сюда приходит любой — сессии нужен `androidx.media3.common.Player`, и ей всё
 * равно, что звук делает Rust. Нажатия с уведомления и с экрана блокировки — обычные
 * приказы плееру. А кнопки гарнитуры разбираются здесь руками: см. [onMediaButtonEvent].
 * Своя команда одна — закрыть приложение совсем.
 */
@UnstableApi
object MediaSessionConfigurator {

    /**
     * `repository` нужен для кнопок гарнитуры: решение «играть или остановить» принимается
     * по состоянию, которое считает Rust, а не по тому, что успела запомнить media3.
     */
    fun createMediaSession(
        service: SongService,
        player: Player,
        repository: SongsRepository,
    ): MediaSession {
        val closeCommand = SessionCommand(Keywords.CloseAction.value, Bundle.EMPTY)
        val close = CommandButton.Builder()
            .setSessionCommand(closeCommand)
            .setIconResId(android.R.drawable.ic_menu_close_clear_cancel)
            .setDisplayName(Keywords.CloseAction.value)
            .setEnabled(true)
            .build()

        return MediaSession.Builder(service, player)
            .setCallback(object : MediaSession.Callback {
                override fun onConnect(
                    session: MediaSession,
                    controllerInfo: MediaSession.ControllerInfo
                ): MediaSession.ConnectionResult {
                    val connectionResult = super.onConnect(session, controllerInfo)
                    val availableSessionCommands = connectionResult.availableSessionCommands
                        .buildUpon()
                        .add(closeCommand)
                        .build()

                    return MediaSession.ConnectionResult.accept(
                        availableSessionCommands,
                        connectionResult.availablePlayerCommands
                    )
                }

                /**
                 * Кнопка на гарнитуре. У неё одна кнопка и одна и та же команда на каждое
                 * нажатие — замерено: код 127, `KEYCODE_MEDIA_PAUSE`, и в начале игры, и на
                 * остановленной музыке. Поэтому «играть» и «пауза» здесь не разделены:
                 * нажатие переключает, а какое именно из трёх названий пришло — неважно.
                 *
                 * Переключение делает [SongsRepository.togglePlaying], потому что состояние
                 * надо читать прямо у плеера. Ни media3, ни поток `isPlaying` для этого не
                 * годятся: и то и другое — пересказ, отстающий от плеера.
                 *
                 * Код клавиши пишется в журнал: иначе не видно, что присылает гарнитура.
                 */
                override fun onMediaButtonEvent(
                    session: MediaSession,
                    controllerInfo: MediaSession.ControllerInfo,
                    intent: Intent,
                ): Boolean {
                    if (intent.action != Intent.ACTION_MEDIA_BUTTON) {
                        return super.onMediaButtonEvent(session, controllerInfo, intent)
                    }
                    val event = intent.getParcelableExtra<KeyEvent>(Intent.EXTRA_KEY_EVENT)
                        ?: return super.onMediaButtonEvent(session, controllerInfo, intent)
                    if (event.action != KeyEvent.ACTION_DOWN) return true
                    Log.i(
                        Keywords.LogTag.value,
                        "кнопка гарнитуры: код ${event.keyCode}, играем ${repository.isPlaying.value}",
                    )
                    when (event.keyCode) {
                        KeyEvent.KEYCODE_MEDIA_PLAY,
                        KeyEvent.KEYCODE_MEDIA_PAUSE,
                        KeyEvent.KEYCODE_MEDIA_PLAY_PAUSE,
                        -> repository.togglePlaying()

                        KeyEvent.KEYCODE_MEDIA_NEXT -> repository.nextTrack()
                        KeyEvent.KEYCODE_MEDIA_PREVIOUS -> repository.prevTrack()
                        KeyEvent.KEYCODE_MEDIA_STOP -> repository.setPlaying(false)
                    }
                    return true
                }

                override fun onCustomCommand(
                    session: MediaSession,
                    controllerInfo: MediaSession.ControllerInfo,
                    customCommand: SessionCommand,
                    args: Bundle
                ): ListenableFuture<SessionResult> {
                    if (customCommand.customAction == Keywords.CloseAction.value) {
                        service.stopSelf()
                        service.terminateApp()
                        return Futures.immediateFuture(SessionResult(SessionResult.RESULT_SUCCESS))
                    }
                    return super.onCustomCommand(session, controllerInfo, customCommand, args)
                }
            })
            .setId(Keywords.PlayerId.value)
            .setCustomLayout(listOf(close))
            .build()
    }

    fun setupNotificationChannel(service: SongService) {
        val channel = NotificationChannel(
            "128",
            "Фоновый процесс для плееров",
            NotificationManager.IMPORTANCE_LOW
        )
        val manager = service.getSystemService(NotificationManager::class.java)
        manager?.createNotificationChannel(channel)
    }
}
