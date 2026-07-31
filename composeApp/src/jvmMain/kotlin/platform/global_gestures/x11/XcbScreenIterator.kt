package platform.global_gestures.x11

import com.sun.jna.Pointer
import com.sun.jna.Structure

/**
 * `xcb_screen_iterator_t`: указатель на текущий экран и сколько осталось. Экраны в ответе сервера
 * разной длины, поэтому к нужному идут вызовом `xcb_screen_next`, а не арифметикой.
 */
@Structure.FieldOrder("data", "rem", "index")
open class XcbScreenIterator : Structure() {

    @JvmField
    var data: Pointer? = null

    @JvmField
    var rem: Int = 0

    @JvmField
    var index: Int = 0

    class ByValue : XcbScreenIterator(), Structure.ByValue
}
