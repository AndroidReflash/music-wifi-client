package platform

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isAltPressed
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.nativeKeyCode
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import constants.DialogConstantsDp
import dataClasses.Hotkey
import enums.HotkeyAction
import java.awt.event.KeyEvent
import musicwifi.composeapp.generated.resources.Res
import musicwifi.composeapp.generated.resources.player_play_triangle_orange
import musicwifi.composeapp.generated.resources.repeat
import musicwifi.composeapp.generated.resources.shuffle
import musicwifi.composeapp.generated.resources.skip_back
import musicwifi.composeapp.generated.resources.skip_forward
import org.jetbrains.compose.resources.DrawableResource
import org.jetbrains.compose.resources.painterResource
import platform.interfaces.GlobalHotkeys
import platform.interfaces.HotkeySettings
import sealedInterfaces.GlobalHotkeysState
import ui.SpeakerGlyph
import ui.songScreenElements.LabeledSwitch

/**
 * Горячие клавиши: ряд значков, по одному на действие. Нажатие на значок открывает окно, которое
 * ждёт клавишу. Назначенное действие — в цвете темы, свободное — серым.
 *
 * Под рядом — переключатель «вне окна» и то, что с перехватом на самом деле. Состояние перехвата
 * берётся прямо у [GlobalHotkeys], а не через ViewModel: это десктопная вещь, общий код о ней не
 * знает, а рисует его только эта, тоже десктопная, половина.
 */
class HotkeySettingsImpl(private val globalHotkeys: GlobalHotkeys) : HotkeySettings {

    @Composable
    override fun Content(
        color: Color,
        hotkeys: Map<HotkeyAction, Hotkey>,
        onAssign: (HotkeyAction, Hotkey?) -> Unit,
        global: Boolean,
        onGlobal: (Boolean) -> Unit,
    ) {
        var editing by remember { mutableStateOf<HotkeyAction?>(null) }
        val globalState by globalHotkeys.state.collectAsState()

        Text("Горячие клавиши", color = Color.White, style = MaterialTheme.typography.bodyMedium)
        // Без промежутков: девять значков по 36 dp и так занимают почти всю ширину окна, а
        // разделяют глифы их собственные отступы.
        Row(
            verticalAlignment = Alignment.CenterVertically,
            modifier = Modifier.padding(bottom = 5.dp),
        ) {
            HotkeyAction.entries.forEach { action ->
                HotkeyButton(
                    action = action,
                    assigned = action in hotkeys,
                    color = color,
                    onClick = { editing = action },
                )
            }
        }
        LabeledSwitch(
            label = "Вне окна",
            checked = global,
            color = color,
            onCheckedChange = onGlobal,
        )
        globalStateText(globalState)?.let { text ->
            Text(text, color = Color.Gray, modifier = Modifier.padding(bottom = 5.dp))
        }

        editing?.let { action ->
            KeyCaptureDialog(
                color = color,
                current = hotkeys[action],
                onAssign = { hotkey -> onAssign(action, hotkey) },
                onDismiss = { editing = null },
            )
        }
    }
}

/** Модификаторы сами по себе не назначаются: окно ждёт клавишу, которую с ними нажмут. */
private val MODIFIER_KEYS = setOf(
    Key.CtrlLeft, Key.CtrlRight,
    Key.AltLeft, Key.AltRight,
    Key.ShiftLeft, Key.ShiftRight,
    Key.MetaLeft, Key.MetaRight,
)

@Composable
private fun HotkeyButton(
    action: HotkeyAction,
    assigned: Boolean,
    color: Color,
    onClick: () -> Unit,
) {
    val tint = if (assigned) color else Color.Gray
    val modifier = Modifier.size(36.dp).clickable { onClick() }.padding(6.dp)
    when (val icon = iconOf(action)) {
        null -> DrawnGlyph(action = action, color = tint, modifier = modifier)
        else -> Icon(
            painter = painterResource(icon),
            contentDescription = labelOf(action),
            tint = tint,
            modifier = modifier,
        )
    }
}

/**
 * Ждёт нажатие. Esc отменяет, «Убрать» снимает назначение; поэтому сам Esc назначить нельзя.
 * Модификаторы без клавиши не принимаются — окно ждёт дальше.
 */
@Composable
private fun KeyCaptureDialog(
    color: Color,
    current: Hotkey?,
    onAssign: (Hotkey?) -> Unit,
    onDismiss: () -> Unit,
) {
    Dialog(
        onDismissRequest = onDismiss,
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        val focus = remember { FocusRequester() }
        Column(
            modifier = Modifier
                .width(DialogConstantsDp.Width.value)
                .background(Color.Black)
                .border(1.dp, Color.White)
                .focusRequester(focus)
                .focusable()
                .onPreviewKeyEvent { event ->
                    if (event.type != KeyEventType.KeyDown || event.key in MODIFIER_KEYS) {
                        return@onPreviewKeyEvent false
                    }
                    if (event.key != Key.Escape) {
                        onAssign(
                            Hotkey(
                                code = event.key.nativeKeyCode,
                                ctrl = event.isCtrlPressed,
                                alt = event.isAltPressed,
                                shift = event.isShiftPressed,
                            )
                        )
                    }
                    onDismiss()
                    true
                }
                .padding(16.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text(
                "Нажмите любую клавишу/сочетание клавиш",
                color = Color.White,
                style = MaterialTheme.typography.bodyMedium,
            )
            Text("Сейчас: ${current?.let(::hotkeyName) ?: "не назначена"}", color = color)
            Text("Esc — отмена", color = Color.Gray)
            Text(
                "Убрать",
                color = Color.White,
                modifier = Modifier
                    .border(1.dp, color)
                    .clickable {
                        onAssign(null)
                        onDismiss()
                    }
                    .padding(horizontal = 12.dp, vertical = 5.dp),
            )
        }
        // Без фокуса окно клавиш не услышит: фокус ставится, когда содержимое уже на месте.
        LaunchedEffect(Unit) {
            focus.requestFocus()
        }
    }
}

/**
 * Что сказать о перехвате под переключателем; `null` — нечего: выключен или перехвачено всё.
 * Отказ показывается, а не проглатывается: иначе включённый переключатель выглядел бы работающим.
 */
private fun globalStateText(state: GlobalHotkeysState): String? = when (state) {
    GlobalHotkeysState.Off -> null
    is GlobalHotkeysState.Unavailable -> state.reason
    is GlobalHotkeysState.Active -> state.rejected.takeIf { it.isNotEmpty() }?.let { rejected ->
        "Не перехвачены, работают только в окне: " +
            rejected.sortedBy { it.ordinal }.joinToString { labelOf(it) }
    }
}

/** Нажатие словами: модификаторы, потом клавиша так, как её называет AWT. */
private fun hotkeyName(hotkey: Hotkey): String =
    listOfNotNull(
        "Ctrl".takeIf { hotkey.ctrl },
        "Alt".takeIf { hotkey.alt },
        "Shift".takeIf { hotkey.shift },
        KeyEvent.getKeyText(hotkey.code),
    ).joinToString("+")

/**
 * Значки те же, что у кнопок управления: по ним действие и узнают. У перемотки и громкости кнопок
 * нет, и значки у них рисованные — отсюда `null`.
 */
private fun iconOf(action: HotkeyAction): DrawableResource? = when (action) {
    HotkeyAction.SHUFFLE -> Res.drawable.shuffle
    HotkeyAction.PREVIOUS -> Res.drawable.skip_back
    HotkeyAction.PLAY_PAUSE -> Res.drawable.player_play_triangle_orange
    HotkeyAction.NEXT -> Res.drawable.skip_forward
    HotkeyAction.REPEAT -> Res.drawable.repeat
    HotkeyAction.REWIND, HotkeyAction.FORWARD -> null
    HotkeyAction.VOLUME_DOWN, HotkeyAction.VOLUME_UP -> null
}

/**
 * Рисованные значки. Громкость — динамик ползунка громкости: «тише» с одной волной, «громче» с
 * тремя.
 */
@Composable
private fun DrawnGlyph(action: HotkeyAction, color: Color, modifier: Modifier) {
    when (action) {
        HotkeyAction.VOLUME_DOWN -> SpeakerGlyph(color = color, modifier = modifier, waves = 1)
        HotkeyAction.VOLUME_UP -> SpeakerGlyph(color = color, modifier = modifier, waves = 3)
        HotkeyAction.REWIND, HotkeyAction.FORWARD ->
            SeekGlyph(forward = action == HotkeyAction.FORWARD, color = color, modifier = modifier)
        // У остальных значок из ресурсов, сюда они не попадают.
        else -> Unit
    }
}

private fun labelOf(action: HotkeyAction): String = when (action) {
    HotkeyAction.SHUFFLE -> "Перемешать"
    HotkeyAction.PREVIOUS -> "Предыдущий"
    HotkeyAction.PLAY_PAUSE -> "Воспроизведение"
    HotkeyAction.NEXT -> "Следующий"
    HotkeyAction.REPEAT -> "Повторить"
    HotkeyAction.REWIND -> "Назад на 5 секунд"
    HotkeyAction.FORWARD -> "Вперёд на 5 секунд"
    HotkeyAction.VOLUME_DOWN -> "Тише"
    HotkeyAction.VOLUME_UP -> "Громче"
}

/**
 * Перемотка: два уголка остриём в сторону движения. Готового значка в ресурсах нет; нарисован
 * прямыми отрезками, как и остальные рисованные значки, — без кривых на глаз.
 */
@Composable
private fun SeekGlyph(forward: Boolean, color: Color, modifier: Modifier = Modifier) {
    Canvas(modifier) {
        val thin = 1.5.dp.toPx()
        val w = size.width
        val h = size.height
        listOf(0.2f, 0.5f).forEach { from ->
            val back = if (forward) w * from else w * (1f - from)
            val tip = if (forward) back + w * 0.3f else back - w * 0.3f
            drawLine(color, Offset(back, h * 0.2f), Offset(tip, h * 0.5f), strokeWidth = thin)
            drawLine(color, Offset(tip, h * 0.5f), Offset(back, h * 0.8f), strokeWidth = thin)
        }
    }
}
