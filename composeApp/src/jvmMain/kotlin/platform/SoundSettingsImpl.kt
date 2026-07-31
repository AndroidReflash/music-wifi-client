package platform

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import platform.interfaces.SoundSettings
import ui.DeviceRow
import ui.VolumeSlider

/**
 * Выбор устройства вывода и громкость приложения.
 *
 * Список устройств спрашивается заново каждый раз, когда окно настроек открывается: устройства
 * подключают и отключают, и показывать то, что было при запуске, значит показывать неправду.
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
    ) {
        LaunchedEffect(Unit) {
            onRefresh()
        }

        Text("Вывод звука", color = Color.White, style = MaterialTheme.typography.bodyMedium)
        Column(
            // Высота ограничена: сколько имён отдаст система, заранее неизвестно, а окно
            // настроек своей высоты не меняет.
            modifier = Modifier.fillMaxWidth(0.9f)
                .heightIn(max = 120.dp)
                .verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(2.dp),
        ) {
            DeviceRow(
                label = "Системное",
                selected = device.isBlank(),
                color = color,
                onPick = { onPickDevice("") },
            )
            devices.forEach { name ->
                DeviceRow(
                    label = name,
                    selected = name == device,
                    color = color,
                    onPick = { onPickDevice(name) },
                )
            }
        }

        VolumeSlider(color = color, volume = volume, onVolume = onVolume)
    }
}
