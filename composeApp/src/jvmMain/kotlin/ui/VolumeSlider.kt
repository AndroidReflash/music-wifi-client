package ui

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp

/**
 * Громкость. Ползунок устроен как полоса перемотки: пока его держат, он идёт за рукой и значение
 * снаружи не слушает, а на отпускании отдаёт своё.
 *
 * Значит и звук меняется на отпускании, а не по ходу движения: громкость хранится в настройках, а
 * писать их на каждый пиксель — это запись на диск десятки раз за одно движение.
 */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun VolumeSlider(
    color: Color,
    volume: Float,
    onVolume: (Float) -> Unit,
) {
    var position by remember { mutableFloatStateOf(volume) }
    var holding by remember { mutableStateOf(false) }

    LaunchedEffect(volume) {
        if (!holding) {
            position = volume
        }
    }

    Row(
        modifier = Modifier.fillMaxWidth(0.9f),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        SpeakerGlyph(color = Color.White, modifier = Modifier.size(20.dp))
        Spacer(modifier = Modifier.width(8.dp))
        Slider(
            value = position,
            onValueChange = {
                holding = true
                position = it
            },
            onValueChangeFinished = {
                holding = false
                onVolume(position)
            },
            thumb = {
                Spacer(
                    modifier = Modifier
                        .size(24.dp)
                        .drawBehind {
                            drawCircle(
                                color = color,
                                radius = 6.dp.toPx(),
                                center = center
                            )
                        }
                )
            },
            colors = SliderDefaults.colors(
                thumbColor = color,
                activeTrackColor = color,
                inactiveTrackColor = Color.Gray,
            ),
            track = { sliderState ->
                SliderDefaults.Track(
                    sliderState = sliderState,
                    modifier = Modifier.height(2.dp),
                    thumbTrackGapSize = 0.dp,
                    trackInsideCornerSize = 0.dp,
                    drawStopIndicator = null,
                    colors = SliderDefaults.colors(
                        activeTrackColor = color,
                        inactiveTrackColor = Color.Gray.copy(alpha = 0.5f)
                    )
                )
            },
            modifier = Modifier.weight(1f),
        )
    }
}