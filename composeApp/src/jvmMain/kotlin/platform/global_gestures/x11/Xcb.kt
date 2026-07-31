package platform.global_gestures.x11

import com.sun.jna.Library
import com.sun.jna.Pointer
import com.sun.jna.ptr.IntByReference

/**
 * Функции `libxcb`, которые нужны перехвату. Имена и порядок параметров — как в `xcb/xproto.h` и
 * `xcb/xcb.h`. Беззнаковые целые C идут знаковыми того же размера: JNA передаёт биты как есть.
 */
internal interface Xcb : Library {

    /**
     * `displayname` = `null` — брать `DISPLAY`. Не возвращает `NULL`: ошибку сообщает
     * [xcb_connection_has_error], а подключение всё равно надо закрыть.
     */
    fun xcb_connect(displayname: String?, screenp: IntByReference): Pointer

    fun xcb_connection_has_error(c: Pointer): Int

    fun xcb_disconnect(c: Pointer)

    fun xcb_flush(c: Pointer): Int

    /** Ответ сервера при подключении. Принадлежит подключению, освобождать не надо. */
    fun xcb_get_setup(c: Pointer): Pointer

    fun xcb_setup_roots_iterator(setup: Pointer): XcbScreenIterator.ByValue

    fun xcb_screen_next(i: XcbScreenIterator)

    fun xcb_generate_id(c: Pointer): Int

    fun xcb_create_window(
        c: Pointer,
        depth: Byte,
        wid: Int,
        parent: Int,
        x: Short,
        y: Short,
        width: Short,
        height: Short,
        borderWidth: Short,
        windowClass: Short,
        visual: Int,
        valueMask: Int,
        valueList: Pointer?,
    ): XcbCookie.ByValue

    fun xcb_get_keyboard_mapping(c: Pointer, firstKeycode: Byte, count: Byte): XcbCookie.ByValue

    fun xcb_get_keyboard_mapping_reply(c: Pointer, cookie: XcbCookie.ByValue, e: Pointer?): Pointer?

    fun xcb_get_keyboard_mapping_keysyms(reply: Pointer): Pointer

    fun xcb_get_keyboard_mapping_keysyms_length(reply: Pointer): Int

    fun xcb_get_modifier_mapping(c: Pointer): XcbCookie.ByValue

    fun xcb_get_modifier_mapping_reply(c: Pointer, cookie: XcbCookie.ByValue, e: Pointer?): Pointer?

    /** Восемь строк по `keycodes_per_modifier` keycode: Shift, Lock, Control, Mod1…Mod5. */
    fun xcb_get_modifier_mapping_keycodes(reply: Pointer): Pointer

    /** «Проверяемый» захват: ошибка приходит в [xcb_request_check], а не в общую очередь. */
    fun xcb_grab_key_checked(
        c: Pointer,
        ownerEvents: Byte,
        grabWindow: Int,
        modifiers: Short,
        key: Byte,
        pointerMode: Byte,
        keyboardMode: Byte,
    ): XcbCookie.ByValue

    fun xcb_ungrab_key(c: Pointer, key: Byte, grabWindow: Int, modifiers: Short): XcbCookie.ByValue

    /** Ждёт, пока сервер обработает запрос. `null` — без ошибки; иначе ошибка, её освободить. */
    fun xcb_request_check(c: Pointer, cookie: XcbCookie.ByValue): Pointer?

    fun xcb_send_event(
        c: Pointer,
        propagate: Byte,
        destination: Int,
        eventMask: Int,
        event: Pointer,
    ): XcbCookie.ByValue

    /** Блокируется до события. `null` — подключение оборвалось. Событие надо освободить. */
    fun xcb_wait_for_event(c: Pointer): Pointer?
}
