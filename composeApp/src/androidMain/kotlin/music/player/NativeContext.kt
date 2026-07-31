package music.player

import android.content.Context

// Отдаёт Rust указатели на виртуальную машину и на Context: без них oboe,
// через который работает cpal, падает на опросе параметров устройства.
object NativeContext {

    private var installed = false

    fun install(context: Context): Boolean {
        if (!installed) {
            System.loadLibrary("player_core")
            installed = initialize(context.applicationContext)
        }
        return installed
    }

    private external fun initialize(context: Context): Boolean
}
