package platform

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import platform.interfaces.SoundSettings

/**
 * На телефоне этих настроек нет, и это не заглушка, которая делает вид.
 *
 * Маршрутом звука на Android распоряжается система: подключённые наушники она подхватывает сама,
 * это проверено. Громкость там тоже системная — кнопками на корпусе, и своя поверх неё означала бы
 * две ручки на одно дело. Поэтому здесь пусто, и [onRefresh] не зовётся: перечисление устройств на
 * Android идёт через JNI и стоит времени.
 */
class SoundSettingsImpl : SoundSettings {

    @Composable
    override fun Content(
        color: Color,
        devices: List<String>,
        device: String,
        volume: Float,
        onRefresh: () -> Unit,
        onPickDevice: (String) -> Unit,
        onVolume: (Float) -> Unit,
    ) = Unit
}
