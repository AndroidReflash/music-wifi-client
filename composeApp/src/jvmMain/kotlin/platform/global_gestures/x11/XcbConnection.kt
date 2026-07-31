package platform.global_gestures.x11

import com.sun.jna.Memory
import com.sun.jna.Native
import com.sun.jna.Pointer
import com.sun.jna.ptr.IntByReference
import constants.XcbConstantsInts
import sealedInterfaces.XcbEvent

/**
 * Своё подключение к X-серверу, отдельное от подключения AWT, через XCB (`libxcb`) — библиотеку, на
 * которой стоит и сам Xlib.
 *
 * Почему не Xlib. Отказ в захвате сочетания X присылает не ответом на запрос, а ошибкой позже, и
 * Xlib отдаёт такие ошибки единому на весь процесс обработчику (`XSetErrorHandler`). Этот
 * обработчик уже поставил AWT: свой пришлось бы ставить поверх чужого, а AWT при запуске ставит
 * свой поверх нашего — кто победит, решал бы порядок запуска. XCB у «проверяемого» запроса отдаёт
 * ошибку тому, кто спросил ([Xcb.xcb_request_check]), и общий обработчик не трогает вовсе. Второе:
 * подключение Xlib нельзя трогать из двух потоков без `XInitThreads`, а тот обязан быть первым
 * вызовом Xlib в процессе — AWT к этому времени уже работает. Подключение XCB потокобезопасно:
 * захват зовётся из потока настроек, события читает свой поток, и всё это — одно подключение.
 *
 * Открывается [open]; не открылось — исключение с причиной, которую видит пользователь.
 */
internal class XcbConnection private constructor(
    private val xcb: Xcb,
    private val connection: Pointer,
    private val root: Int,
    private val wakeWindow: Int,
    private val minKeycode: Int,
    private val keysymsPerKeycode: Int,
    private val keysyms: IntArray,
    modifierKeycodes: List<Int>,
    keycodesPerModifier: Int,
) {

    /**
     * Маска, которую добавляет включённый Num Lock; 0 — Num Lock не посажен ни на один
     * модификатор, и учитывать нечего.
     */
    val numLockMask: Int = modifierMaskOf(
        setOf(XcbConstantsInts.KeysymNumLock.value),
        modifierKeycodes,
        keycodesPerModifier,
    ) ?: 0

    /**
     * Маска Alt; `null` — Alt не посажен ни на один модификатор, и сочетание с Alt X не отличит
     * от сочетания без него.
     */
    val altMask: Int? = modifierMaskOf(
        setOf(XcbConstantsInts.KeysymAltLeft.value, XcbConstantsInts.KeysymAltRight.value),
        modifierKeycodes,
        keycodesPerModifier,
    )

    /**
     * Keycode — номер физической клавиши — той, на которой есть [keysym] в любой раскладке и на
     * любом уровне; `null` — такой клавиши на клавиатуре нет.
     */
    fun keycodeOf(keysym: Int): Int? {
        val index = keysyms.indexOf(keysym)
        return if (index < 0) null else minKeycode + index / keysymsPerKeycode
    }

    /**
     * Маска модификатора Mod1…Mod5, на который система посадила клавишу с одним из [wanted]
     * keysym; `null` — ни на один. Так же номер Alt находит и сам AWT: обычно Alt — Mod1, а Num
     * Lock — Mod2, но раскладку модификаторов задаёт система, а не протокол.
     */
    private fun modifierMaskOf(
        wanted: Set<Int>,
        modifierKeycodes: List<Int>,
        keycodesPerModifier: Int,
    ): Int? {
        val first = XcbConstantsInts.FirstFreeModifier.value
        val row = (first until XcbConstantsInts.ModifierRows.value).firstOrNull { row ->
            modifierKeycodes
                .subList(row * keycodesPerModifier, (row + 1) * keycodesPerModifier)
                .any { keycode -> keysymsOf(keycode).any { keysym -> keysym in wanted } }
        } ?: return null
        return 1 shl row
    }

    /** Все keysym клавиши, по раскладкам и уровням; пусто — такого keycode нет. */
    private fun keysymsOf(keycode: Int): List<Int> {
        val first = (keycode - minKeycode) * keysymsPerKeycode
        if (first < 0 || first >= keysyms.size) {
            return emptyList()
        }
        return keysyms.asList().subList(first, first + keysymsPerKeycode)
    }

    /**
     * Захватывает клавишу на корневом окне — то есть у всех окон сразу — с каждым набором
     * [modifiers]. Удалось всё — `true`; хоть один набор занят другой программой — снимает
     * удавшиеся и отвечает `false`: сочетание, работающее только при выключенном Num Lock, хуже
     * честного отказа.
     *
     * Запросы уходят все, потом проверяются: проверка ждёт сервер, и так он ждётся один раз.
     */
    fun grab(keycode: Int, modifiers: List<Int>): Boolean {
        val cookies = modifiers.map { mask ->
            xcb.xcb_grab_key_checked(
                c = connection,
                ownerEvents = 0,
                grabWindow = root,
                modifiers = mask.toShort(),
                key = keycode.toByte(),
                pointerMode = XcbConstantsInts.GrabModeAsync.value.toByte(),
                keyboardMode = XcbConstantsInts.GrabModeAsync.value.toByte(),
            )
        }
        val granted = cookies.map { cookie ->
            val error = xcb.xcb_request_check(connection, cookie)
            error?.let(::free)
            error == null
        }
        if (granted.all { it }) {
            return true
        }
        modifiers.zip(granted).filter { (_, ok) -> ok }.forEach { (mask, _) ->
            xcb.xcb_ungrab_key(connection, keycode.toByte(), root, mask.toShort())
        }
        xcb.xcb_flush(connection)
        return false
    }

    /** Снимает все свои захваты. Чужие не трогает: снять можно только то, что захватил сам. */
    fun ungrabAll() {
        xcb.xcb_ungrab_key(
            connection,
            XcbConstantsInts.AnyKey.value.toByte(),
            root,
            XcbConstantsInts.AnyModifier.value.toShort(),
        )
        xcb.xcb_flush(connection)
    }

    /** Ждёт следующее событие. `null` — подключение оборвалось. */
    fun nextEvent(): XcbEvent? {
        val event = xcb.xcb_wait_for_event(connection) ?: return null
        try {
            val keycode = event.byteAt(XcbConstantsInts.KeyEventKeycode.value)
            return when (event.byteAt(0) and XcbConstantsInts.ResponseTypeMask.value) {
                XcbConstantsInts.KeyPress.value -> XcbEvent.KeyPress(
                    keycode = keycode,
                    state = event.shortAt(XcbConstantsInts.KeyEventState.value),
                )
                XcbConstantsInts.KeyRelease.value -> XcbEvent.KeyRelease(keycode)
                XcbConstantsInts.ClientMessage.value ->
                    if (event.intAt(XcbConstantsInts.ClientMessageWindow.value) == wakeWindow) {
                        XcbEvent.Wake
                    } else {
                        XcbEvent.Other
                    }
                else -> XcbEvent.Other
            }
        } finally {
            free(event)
        }
    }

    /**
     * Будит поток, ждущий в [nextEvent]: шлёт событие своему невидимому окну. Событие без маски
     * получателей X отдаёт тому, кто окно создал, — то есть этому же подключению.
     */
    fun wake() {
        val event = Memory(XcbConstantsInts.EventSize.value.toLong()).apply {
            clear()
            setByte(0, XcbConstantsInts.ClientMessage.value.toByte())
            setByte(
                XcbConstantsInts.ClientMessageFormatOffset.value.toLong(),
                XcbConstantsInts.ClientMessageFormat.value.toByte(),
            )
            setInt(XcbConstantsInts.ClientMessageWindow.value.toLong(), wakeWindow)
        }
        xcb.xcb_send_event(connection, 0, wakeWindow, 0, event)
        xcb.xcb_flush(connection)
    }

    /** После этого подключением не пользуется никто, в том числе поток событий. */
    fun disconnect() {
        xcb.xcb_disconnect(connection)
    }

    companion object {

        /**
         * Подключается к серверу из `DISPLAY` и готовит всё, что нужно захвату: корневое окно,
         * раскладку клавиатуры и модификаторов, автоповтор без отпусканий и окно для пробуждения.
         *
         * Библиотеки — по именам с номером версии: так они называются в любой системе, а
         * `libxcb.so` без номера есть только там, где поставлены заголовки для сборки.
         */
        fun open(): XcbConnection {
            val xcb = Native.load("libxcb.so.1", Xcb::class.java)
            val xkb = Native.load("libxcb-xkb.so.1", XcbXkb::class.java)
            val screen = IntByReference()
            val connection = xcb.xcb_connect(null, screen)
            try {
                check(xcb.xcb_connection_has_error(connection) == 0) {
                    "не удалось подключиться к X-серверу"
                }
                enableDetectableAutoRepeat(xkb, connection)
                val setup = xcb.xcb_get_setup(connection)
                val root = rootOf(xcb, setup, screen.value)
                val minKeycode = setup.byteAt(XcbConstantsInts.SetupMinKeycode.value)
                val maxKeycode = setup.byteAt(XcbConstantsInts.SetupMaxKeycode.value)
                val mapping = xcb.xcb_get_keyboard_mapping_reply(
                    connection,
                    xcb.xcb_get_keyboard_mapping(
                        connection,
                        minKeycode.toByte(),
                        (maxKeycode - minKeycode + 1).toByte(),
                    ),
                    null,
                ) ?: error("X-сервер не отдал раскладку клавиатуры")
                val keysymsPerKeycode =
                    mapping.byteAt(XcbConstantsInts.MappingKeysymsPerKeycode.value)
                val keysyms = xcb.xcb_get_keyboard_mapping_keysyms(mapping)
                    .getIntArray(0, xcb.xcb_get_keyboard_mapping_keysyms_length(mapping))
                free(mapping)
                val modifiers = xcb.xcb_get_modifier_mapping_reply(
                    connection,
                    xcb.xcb_get_modifier_mapping(connection),
                    null,
                ) ?: error("X-сервер не отдал раскладку модификаторов")
                val keycodesPerModifier =
                    modifiers.byteAt(XcbConstantsInts.ModifierMappingKeycodesPerModifier.value)
                val modifierKeycodes = xcb.xcb_get_modifier_mapping_keycodes(modifiers)
                    .getByteArray(0, XcbConstantsInts.ModifierRows.value * keycodesPerModifier)
                    .map { keycode -> keycode.toInt() and 0xff }
                free(modifiers)
                val wakeWindow = xcb.xcb_generate_id(connection)
                xcb.xcb_create_window(
                    c = connection,
                    depth = 0,
                    wid = wakeWindow,
                    parent = root,
                    x = 0,
                    y = 0,
                    width = 1,
                    height = 1,
                    borderWidth = 0,
                    windowClass = XcbConstantsInts.WindowClassInputOnly.value.toShort(),
                    visual = 0,
                    valueMask = 0,
                    valueList = null,
                )
                xcb.xcb_flush(connection)
                return XcbConnection(
                    xcb = xcb,
                    connection = connection,
                    root = root,
                    wakeWindow = wakeWindow,
                    minKeycode = minKeycode,
                    keysymsPerKeycode = keysymsPerKeycode,
                    keysyms = keysyms,
                    modifierKeycodes = modifierKeycodes,
                    keycodesPerModifier = keycodesPerModifier,
                )
            } catch (failure: Throwable) {
                // Подключение XCB создаётся и при ошибке, и освобождать его надо в любом случае.
                xcb.xcb_disconnect(connection)
                throw failure
            }
        }

        /**
         * Без этого X присылает автоповтор парой «отпущена — нажата», и удержание не отличить от
         * частых нажатий: «играть/пауза» переключалось бы раз за разом. С этим флагом повтор
         * приходит нажатием без отпускания.
         */
        private fun enableDetectableAutoRepeat(xkb: XcbXkb, connection: Pointer) {
            val extension = xkb.xcb_xkb_use_extension_reply(
                connection,
                xkb.xcb_xkb_use_extension(connection, 1, 0),
                null,
            ) ?: error("X-сервер не ответил на запрос XKB")
            val supported = extension.byteAt(XcbConstantsInts.XkbUseExtensionSupported.value) != 0
            free(extension)
            check(supported) { "у X-сервера нет расширения XKB" }

            val flag = XcbConstantsInts.XkbDetectableAutoRepeat.value
            val flags = xkb.xcb_xkb_per_client_flags_reply(
                connection,
                xkb.xcb_xkb_per_client_flags(
                    c = connection,
                    deviceSpec = XcbConstantsInts.XkbUseCoreKeyboard.value.toShort(),
                    change = flag,
                    value = flag,
                    ctrlsToChange = 0,
                    autoCtrls = 0,
                    autoCtrlsValues = 0,
                ),
                null,
            ) ?: error("X-сервер не ответил на запрос флагов XKB")
            val enabled = flags.intAt(XcbConstantsInts.XkbPerClientFlagsValue.value) and flag != 0
            free(flags)
            check(enabled) { "X-сервер не умеет сообщать автоповтор без отпускания" }
        }

        /** Корневое окно экрана [screen] — того, что назван в `DISPLAY` после точки. */
        private fun rootOf(xcb: Xcb, setup: Pointer, screen: Int): Int {
            // Итератор по значению переписывается в обычный: `xcb_screen_next` двигает его по
            // указателю, а JNA передаёт структуру по значению, если её класс — `ByValue`.
            val first = xcb.xcb_setup_roots_iterator(setup)
            val screens = XcbScreenIterator().apply {
                data = first.data
                rem = first.rem
                index = first.index
            }
            repeat(screen) { xcb.xcb_screen_next(screens) }
            val data = screens.data?.takeIf { screens.rem > 0 }
                ?: error("на X-сервере нет экрана $screen")
            return data.intAt(XcbConstantsInts.ScreenRoot.value)
        }
    }
}

/** Беззнаковый байт по смещению. */
private fun Pointer.byteAt(offset: Int): Int = getByte(offset.toLong()).toInt() and 0xff

/** Беззнаковое 16-битное по смещению. */
private fun Pointer.shortAt(offset: Int): Int = getShort(offset.toLong()).toInt() and 0xffff

/** 32-битное по смещению. */
private fun Pointer.intAt(offset: Int): Int = getInt(offset.toLong())

/** Ответы и события XCB выделены `malloc` и освобождаются `free`. */
private fun free(pointer: Pointer) {
    Native.free(Pointer.nativeValue(pointer))
}
