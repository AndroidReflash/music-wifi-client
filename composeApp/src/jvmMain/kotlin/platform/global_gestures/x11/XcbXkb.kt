package platform.global_gestures.x11

import com.sun.jna.Library
import com.sun.jna.Pointer

/**
 * Функции `libxcb-xkb` — расширения XKB, которое в X отвечает за клавиатуру. Нужно одно: чтобы
 * автоповтор приходил нажатием без отпускания перед ним, и удержание отличалось от новых нажатий.
 */
internal interface XcbXkb : Library {

    /** Включает XKB для подключения; без этого остальные запросы XKB сервер не примет. */
    fun xcb_xkb_use_extension(c: Pointer, wantedMajor: Short, wantedMinor: Short): XcbCookie.ByValue

    fun xcb_xkb_use_extension_reply(c: Pointer, cookie: XcbCookie.ByValue, e: Pointer?): Pointer?

    fun xcb_xkb_per_client_flags(
        c: Pointer,
        deviceSpec: Short,
        change: Int,
        value: Int,
        ctrlsToChange: Int,
        autoCtrls: Int,
        autoCtrlsValues: Int,
    ): XcbCookie.ByValue

    fun xcb_xkb_per_client_flags_reply(c: Pointer, cookie: XcbCookie.ByValue, e: Pointer?): Pointer?
}
