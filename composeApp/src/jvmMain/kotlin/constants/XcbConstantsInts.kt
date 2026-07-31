package constants

/**
 * Числа протокола X11 и раскладка структур XCB, у которых тип `Int`. Взяты из заголовков
 * `xcb/xproto.h` и `xcb/xkb.h`; смещения — в байтах от начала структуры, какой её отдаёт XCB.
 */
sealed class XcbConstantsInts(val value: Int) {

    /** Тип события «клавиша нажата» — первый байт события. */
    data object KeyPress : XcbConstantsInts(2)

    /** Тип события «клавиша отпущена». */
    data object KeyRelease : XcbConstantsInts(3)

    /** Тип события, которое клиент шлёт окну сам, — им будится поток событий. */
    data object ClientMessage : XcbConstantsInts(33)

    /**
     * Первый байт события без старшего бита. Старший бит ставится у событий, присланных через
     * `SendEvent`, — к типу он не относится.
     */
    data object ResponseTypeMask : XcbConstantsInts(0x7f)

    data object ShiftMask : XcbConstantsInts(1)

    /** Caps Lock: для X это модификатор, и зажатым он входит в сочетание. */
    data object LockMask : XcbConstantsInts(2)

    data object ControlMask : XcbConstantsInts(4)

    /**
     * Номер строки Mod1 в раскладке модификаторов; за ней Mod2…Mod5, последняя — седьмая. На
     * какой из них Alt и Num Lock, решает система — это спрашивается у сервера. Маска строки —
     * единица, сдвинутая на её номер.
     */
    data object FirstFreeModifier : XcbConstantsInts(3)

    /** Строк в раскладке модификаторов: Shift, Lock, Control, Mod1…Mod5. */
    data object ModifierRows : XcbConstantsInts(8)

    /** Keysym `Num_Lock`. */
    data object KeysymNumLock : XcbConstantsInts(0xff7f)

    /** Keysym `Alt_L`. */
    data object KeysymAltLeft : XcbConstantsInts(0xffe9)

    /** Keysym `Alt_R`. */
    data object KeysymAltRight : XcbConstantsInts(0xffea)

    /** Для `UngrabKey`: любые модификаторы. */
    data object AnyModifier : XcbConstantsInts(0x8000)

    /** Для `UngrabKey`: любая клавиша. */
    data object AnyKey : XcbConstantsInts(0)

    /** Захват не замораживает ввод: остальные программы получают события как обычно. */
    data object GrabModeAsync : XcbConstantsInts(1)

    /** Класс окна «только ввод»: невидимое, ничего не рисует. */
    data object WindowClassInputOnly : XcbConstantsInts(2)

    /** XKB: устройство — основная клавиатура. */
    data object XkbUseCoreKeyboard : XcbConstantsInts(0x100)

    /** XKB, флаг подключения: автоповтор приходит нажатием без отпускания перед ним. */
    data object XkbDetectableAutoRepeat : XcbConstantsInts(1)

    /** Длина любого события X. */
    data object EventSize : XcbConstantsInts(32)

    /** `ClientMessage`: данные — 32-битными словами. */
    data object ClientMessageFormat : XcbConstantsInts(32)

    /** `xcb_setup_t.min_keycode`. */
    data object SetupMinKeycode : XcbConstantsInts(34)

    /** `xcb_setup_t.max_keycode`. */
    data object SetupMaxKeycode : XcbConstantsInts(35)

    /** `xcb_screen_t.root`. */
    data object ScreenRoot : XcbConstantsInts(0)

    /** `xcb_key_press_event_t.detail` — keycode, один байт. */
    data object KeyEventKeycode : XcbConstantsInts(1)

    /** `xcb_key_press_event_t.state` — зажатые модификаторы, два байта. */
    data object KeyEventState : XcbConstantsInts(28)

    /** `xcb_client_message_event_t.format`. */
    data object ClientMessageFormatOffset : XcbConstantsInts(1)

    /** `xcb_client_message_event_t.window`. */
    data object ClientMessageWindow : XcbConstantsInts(4)

    /** `xcb_get_keyboard_mapping_reply_t.keysyms_per_keycode`. */
    data object MappingKeysymsPerKeycode : XcbConstantsInts(1)

    /** `xcb_get_modifier_mapping_reply_t.keycodes_per_modifier`. */
    data object ModifierMappingKeycodesPerModifier : XcbConstantsInts(1)

    /** `xcb_xkb_use_extension_reply_t.supported`. */
    data object XkbUseExtensionSupported : XcbConstantsInts(1)

    /** `xcb_xkb_per_client_flags_reply_t.value` — какие флаги в итоге стоят. */
    data object XkbPerClientFlagsValue : XcbConstantsInts(12)
}
