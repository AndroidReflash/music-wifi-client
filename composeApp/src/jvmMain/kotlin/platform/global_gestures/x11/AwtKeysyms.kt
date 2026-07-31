package platform.global_gestures.x11

import java.awt.event.KeyEvent

/**
 * Keysym X11 — имя символа на клавише — для кода клавиши AWT, которым хранится назначение;
 * `null` — перевода нет, и такое назначение не перехватывается. AWT своего перевода наружу не
 * отдаёт. Буквы, цифры и пробел совпадают с keysym как есть — и там и там это коды ASCII;
 * функциональные клавиши и цифры цифрового блока идут подряд; остальное — таблицей.
 */
internal fun keysymOf(awtCode: Int): Int? = when (awtCode) {
    in KeyEvent.VK_A..KeyEvent.VK_Z, in KeyEvent.VK_0..KeyEvent.VK_9 -> awtCode
    in KeyEvent.VK_F1..KeyEvent.VK_F12 -> XK_F1 + awtCode - KeyEvent.VK_F1
    in KeyEvent.VK_F13..KeyEvent.VK_F24 -> XK_F13 + awtCode - KeyEvent.VK_F13
    in KeyEvent.VK_NUMPAD0..KeyEvent.VK_NUMPAD9 -> XK_KP_0 + awtCode - KeyEvent.VK_NUMPAD0
    else -> KEYSYMS[awtCode]
}

private const val XK_F1 = 0xffbe
private const val XK_F13 = 0xffca
private const val XK_KP_0 = 0xffb0

/** Значения — из `X11/keysymdef.h`, имя keysym — в комментарии. */
private val KEYSYMS = mapOf(
    KeyEvent.VK_SPACE to 0x0020, // space
    KeyEvent.VK_ENTER to 0xff0d, // Return
    KeyEvent.VK_BACK_SPACE to 0xff08, // BackSpace
    KeyEvent.VK_TAB to 0xff09, // Tab
    KeyEvent.VK_PAUSE to 0xff13, // Pause
    KeyEvent.VK_SCROLL_LOCK to 0xff14, // Scroll_Lock
    KeyEvent.VK_PRINTSCREEN to 0xff61, // Print
    KeyEvent.VK_INSERT to 0xff63, // Insert
    KeyEvent.VK_DELETE to 0xffff, // Delete
    KeyEvent.VK_HOME to 0xff50, // Home
    KeyEvent.VK_END to 0xff57, // End
    KeyEvent.VK_PAGE_UP to 0xff55, // Prior
    KeyEvent.VK_PAGE_DOWN to 0xff56, // Next
    KeyEvent.VK_LEFT to 0xff51, // Left
    KeyEvent.VK_UP to 0xff52, // Up
    KeyEvent.VK_RIGHT to 0xff53, // Right
    KeyEvent.VK_DOWN to 0xff54, // Down
    KeyEvent.VK_CONTEXT_MENU to 0xff67, // Menu
    KeyEvent.VK_MULTIPLY to 0xffaa, // KP_Multiply
    KeyEvent.VK_ADD to 0xffab, // KP_Add
    KeyEvent.VK_SEPARATOR to 0xffac, // KP_Separator
    KeyEvent.VK_SUBTRACT to 0xffad, // KP_Subtract
    KeyEvent.VK_DECIMAL to 0xffae, // KP_Decimal
    KeyEvent.VK_DIVIDE to 0xffaf, // KP_Divide
    KeyEvent.VK_COMMA to 0x002c, // comma
    KeyEvent.VK_MINUS to 0x002d, // minus
    KeyEvent.VK_PERIOD to 0x002e, // period
    KeyEvent.VK_SLASH to 0x002f, // slash
    KeyEvent.VK_SEMICOLON to 0x003b, // semicolon
    KeyEvent.VK_EQUALS to 0x003d, // equal
    KeyEvent.VK_OPEN_BRACKET to 0x005b, // bracketleft
    KeyEvent.VK_BACK_SLASH to 0x005c, // backslash
    KeyEvent.VK_CLOSE_BRACKET to 0x005d, // bracketright
    KeyEvent.VK_BACK_QUOTE to 0x0060, // grave
    KeyEvent.VK_QUOTE to 0x0027, // apostrophe
)
