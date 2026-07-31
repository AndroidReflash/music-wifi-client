package music

import android.content.Intent
import android.os.Handler
import android.os.Looper
import android.os.Process
import androidx.media3.common.util.UnstableApi
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import music.mediasession.MediaSessionConfigurator
import org.koin.android.ext.android.inject
import platform.RustMediaPlayer
import repository.interfaces.SongsRepository
import kotlin.system.exitProcess

/**
 * Служба переднего плана. Она нужна не плееру, а системе: без неё процесс в фоне
 * прибивают, и поток вывода звука умирает вместе с ним.
 *
 * Воспроизведением она не распоряжается и состояния не хранит — и то и другое у
 * [SongsRepository], который приходит из того же графа зависимостей, что и экран. Дело
 * службы — жить и держать медиасессию.
 */
@UnstableApi
class SongService : MediaSessionService() {

    private val repository: SongsRepository by inject()
    private val serviceJob = SupervisorJob()
    private val scope = CoroutineScope(Dispatchers.Main + serviceJob)
    private var session: MediaSession? = null

    override fun onCreate() {
        super.onCreate()
        MediaSessionConfigurator.setupNotificationChannel(this)
        // Плеер медиасессии живёт на главном потоке: media3 спрашивает его состояние
        // только оттуда.
        val player = RustMediaPlayer(Looper.getMainLooper(), scope, repository, this.applicationContext)
        val created = MediaSessionConfigurator.createMediaSession(this, player, repository)
        session = created
        addSession(created)
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = session

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        super.onStartCommand(intent, flags, startId)
        return START_NOT_STICKY
    }

    /** Приложение убрали из недавних — уходим совсем. */
    override fun onTaskRemoved(rootIntent: Intent?) {
        super.onTaskRemoved(rootIntent)
        release()
        stopForeground(STOP_FOREGROUND_DETACH)
        terminateApp()
    }

    override fun onDestroy() {
        release()
        super.onDestroy()
    }

    private fun release() {
        serviceJob.cancel()
        session?.let { open ->
            open.player.release()
            open.release()
        }
        session = null
    }

    /**
     * Убивает процесс. Так закрывается приложение кнопкой в уведомлении: иначе оно
     * останется висеть без окна и без звука.
     */
    fun terminateApp() {
        Handler(Looper.getMainLooper()).postDelayed({
            Process.killProcess(Process.myPid())
            exitProcess(0)
        }, 200)
    }
}
