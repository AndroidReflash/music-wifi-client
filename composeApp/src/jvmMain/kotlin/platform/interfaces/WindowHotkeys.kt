package platform.interfaces

import androidx.compose.ui.input.key.KeyEvent

/**
 * Горячие клавиши окна: нажатия, пришедшие окну приложения, пока оно в фокусе.
 *
 * Интерфейс лежит в десктопном наборе, а не в общем: окна с клавиатурой есть только здесь, и
 * общий код о нём не знает.
 */
interface WindowHotkeys {

    /**
     * Для `Window(onPreviewKeyEvent = …)`. `true` — нажатие назначено и забрано.
     */
    fun onPreviewKeyEvent(event: KeyEvent): Boolean

    /** Курсор в поле поиска: пока он там, нажатия принадлежат полю. */
    fun setTextEntry(active: Boolean)

    /** Окно потеряло фокус: считать клавиши зажатыми дальше нельзя. */
    fun releaseAll()
}
