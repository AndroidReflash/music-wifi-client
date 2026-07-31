package ui.songScreenElements

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import constants.DialogConstantsDp
import enums.AppTheme
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import dataClasses.Hotkey
import enums.HotkeyAction
import platform.interfaces.HotkeySettings
import platform.interfaces.PanelSettings
import platform.interfaces.SoundSettings
import androidx.compose.ui.window.DialogProperties

@Composable
fun ChangeIpDialog(
    color: Color,
    isLocal: Boolean,
    onSwitch: () -> Unit,
    onDismiss :() -> Unit,
    onConfirmOnline: (String) -> Unit,
    onChooseOfflineFolder: () -> Unit,
    currentIp: String,
    folderAddress: String,
    onThemeSelected: (Color) -> Unit,
    showVisualizer: Boolean,
    showCover: Boolean,
    showCoverImage: Boolean,
    visualizerRing: Boolean,
    coverSmall: Boolean,
    onShowVisualizer: (Boolean) -> Unit,
    onShowCover: (Boolean) -> Unit,
    onShowCoverImage: (Boolean) -> Unit,
    onVisualizerRing: (Boolean) -> Unit,
    onCoverSmall: (Boolean) -> Unit,
    outputDevices: List<String>,
    outputDevice: String,
    volume: Float,
    onRefreshOutputDevices: () -> Unit,
    onPickOutputDevice: (String) -> Unit,
    onVolume: (Float) -> Unit,
    panelDivider: Boolean,
    panelTransparent: Boolean,
    onPanelDivider: (Boolean) -> Unit,
    onPanelTransparent: (Boolean) -> Unit,
    hotkeys: Map<HotkeyAction, Hotkey>,
    onHotkey: (HotkeyAction, Hotkey?) -> Unit,
    hotkeysGlobal: Boolean,
    onHotkeysGlobal: (Boolean) -> Unit,
    autoplay: Boolean,
    onAutoplay: (Boolean) -> Unit,
    soundSettings: SoundSettings,
    panelSettings: PanelSettings,
    hotkeySettings: HotkeySettings,
){
    Dialog(
        onDismissRequest = { onDismiss() },
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ){
        Box(
            Modifier
            .size(width = DialogConstantsDp.Width.value, height = DialogConstantsDp.Height.value)
            .background(Color.Black)
            .border(1.dp, Color.White),
            contentAlignment = Alignment.Center
        ){
            var text by remember { mutableStateOf("") }
            Column (
                // Прокрутка нужна десктопу: там к настройкам добавляется выбор устройства
                // вывода, а высота окна задана долей и своей не меняет. Пока содержимое влезает
                // — а на телефоне оно влезает, — прокрутка не видна и вида не меняет.
                modifier = Modifier.verticalScroll(rememberScrollState()),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(5.dp)
            ) {
                // На телефоне эта половина не рисует ничего: клавиатуры там нет.
                hotkeySettings.Content(
                    color = color,
                    hotkeys = hotkeys,
                    onAssign = onHotkey,
                    global = hotkeysGlobal,
                    onGlobal = onHotkeysGlobal,
                )
                Text("Тема", color = Color.White, style = MaterialTheme.typography.bodyMedium)
                Row(
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    modifier = Modifier.padding(bottom = 5.dp)
                ) {
                    AppTheme.entries.forEach { theme ->
                        Box(
                            modifier = Modifier
                                .size(24.dp)
                                .background(theme.color)
                                .border(
                                    width = if (color == theme.color) 2.dp else 1.dp,
                                    color = Color.White
                                )
                                .clickable {
                                    onThemeSelected(theme.color)
                                }
                        )
                    }
                }

                LabeledSwitch(
                    label = if (!isLocal) "По сети" else "Локально",
                    checked = isLocal,
                    color = color,
                    onCheckedChange = { onSwitch() },
                )
                LabeledSwitch(
                    label = "Автоплей",
                    checked = autoplay,
                    color = color,
                    onCheckedChange = onAutoplay,
                )
                SettingGlyphs(
                    color = color,
                    showVisualizer = showVisualizer,
                    showCover = showCover,
                    showCoverImage = showCoverImage,
                    ring = visualizerRing,
                    small = coverSmall,
                    onShowVisualizer = onShowVisualizer,
                    onShowCover = onShowCover,
                    onShowCoverImage = onShowCoverImage,
                    onRing = onVisualizerRing,
                    onSmall = onCoverSmall,
                )
                // На десктопе эта половина не рисует ничего: панель там ни на чём не лежит.
                panelSettings.Content(
                    color = color,
                    divider = panelDivider,
                    transparent = panelTransparent,
                    onDivider = onPanelDivider,
                    onTransparent = onPanelTransparent,
                )
                // На телефоне эта половина не рисует ничего: маршрутом там распоряжается
                // система, и громкость там системная.
                soundSettings.Content(
                    color = color,
                    devices = outputDevices,
                    device = outputDevice,
                    volume = volume,
                    onRefresh = onRefreshOutputDevices,
                    onPickDevice = onPickOutputDevice,
                    onVolume = onVolume,
                )
                if(!isLocal){
                    Text("Текущий айпи: $currentIp", color = Color.White,
                        modifier =  Modifier.padding(bottom = 10.dp)
                    )
                }else{
                    Text(" ", color = Color.White,
                        modifier =  Modifier.padding(bottom = 10.dp)
                    )
                }
                if(!isLocal){
                    Text("Введите айпи сервера", color = Color.White)
                    Text("(пример: 192.168.100.9:9090)", color = Color.White)
                    BasicTextField(
                        value = text,
                        onValueChange = { text = it },
                        textStyle = TextStyle(color = Color.White),
                        modifier = Modifier
                            .background(Color.Black).fillMaxWidth(0.9f)
                            .border(1.dp, Color.White)
                            .padding(8.dp)
                    )
                    Box(Modifier.border(1.dp, color).clickable{
                        onConfirmOnline(text)
                    }){
                        Text("Применить", color = Color.White, modifier = Modifier.padding(5.dp))
                    }
                }else{
                    Text("Текущая папка: \n $folderAddress", color = Color.White,
                        modifier =  Modifier.padding(bottom = 10.dp)
                    )
                    Box(Modifier.border(1.dp, color).clickable{
                        onChooseOfflineFolder()
                    }){
                        Text("Выбрать папку с треками",
                            color = Color.White,
                            modifier = Modifier.padding(5.dp)
                        )
                    }
                }
            }
        }
    }
}

/**
 * Надпись и переключатель в строку. Три переключателя подряд с одними и теми же цветами — это
 * один вид, а не три похожих. Не закрыт в файле: тем же видом рисуется переключатель глобальных
 * клавиш в десктопной половине настроек.
 */
@Composable
internal fun LabeledSwitch(
    label: String,
    checked: Boolean,
    color: Color,
    onCheckedChange: (Boolean) -> Unit,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp, Alignment.CenterHorizontally),
        modifier = Modifier.fillMaxWidth()
    ) {
        Text(
            text = label,
            color = Color.White,
            style = MaterialTheme.typography.bodyLarge
        )

        Spacer(modifier = Modifier.width(12.dp))

        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange,
            colors = SwitchDefaults.colors(
                checkedThumbColor = Color.White,
                uncheckedThumbColor = Color.White,
                checkedTrackColor = color,
                uncheckedTrackColor = Color.Gray,
                uncheckedBorderColor = Color.Gray
            )
        )
    }
}
