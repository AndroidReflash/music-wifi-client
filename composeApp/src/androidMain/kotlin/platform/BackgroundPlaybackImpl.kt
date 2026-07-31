package platform

import android.content.Context
import android.content.Intent
import music.SongService
import platform.interfaces.BackgroundPlayback

/**
 * Служба переднего плана: пока она жива, жив процесс, а с ним поток вывода звука.
 *
 * Запускается в тот миг, когда пошёл звук, а не при создании графа зависимостей:
 * служба переднего плана обязана показать уведомление почти сразу, а показывать его
 * нечего, пока ничего не играет.
 */
class BackgroundPlaybackImpl(private val context: Context) : BackgroundPlayback {

    override fun start() {
        context.startForegroundService(Intent(context, SongService::class.java))
    }
}
