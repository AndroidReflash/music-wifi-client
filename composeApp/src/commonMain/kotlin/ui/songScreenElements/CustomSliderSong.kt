package ui.songScreenElements

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
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
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomSliderSong(
    color: Color,
    modifier: Modifier = Modifier,
    length: String,
    position: String,
    progress: Float,
    onSeekTo: (Float) -> Unit
){

    val fileSize = "00:00" // для корректного подсчёта ширины места под текст
    val textMeasurer = rememberTextMeasurer()
    val fontSize = 12
    val fixedWidth = with(LocalDensity.current) {
        val style = TextStyle(
            fontSize = fontSize.sp,
        )

        textMeasurer.measure(
            text = AnnotatedString("8"),
            style = style
        ).size.width.toDp()
    }

    var sliderPosition by remember { mutableFloatStateOf(0f) }
    var isUserInteracting by remember { mutableStateOf(false) }

    // следим за обновлением с плеера, только если пользователь не трогает слайдер
    LaunchedEffect(progress) {
        if (!isUserInteracting) {
            sliderPosition = progress
        }
    }

    Row(
        modifier = modifier
            .fillMaxWidth(0.9f).defaultMinSize(minHeight = 60.dp)
        ,
        horizontalArrangement = Arrangement.Center,
        verticalAlignment = Alignment.CenterVertically
    ) {

        // Обводка: при прозрачной панели подписи времени оказываются на картинке обложки.
        OutlinedText(
            text = position,
            color = Color.Gray,
            modifier = modifier.width(fixedWidth*fileSize.length),
            textModifier = Modifier.fillMaxWidth(),
            fontSize = fontSize.sp,
            lineHeight = fontSize.sp*1.2f,
            textAlign = TextAlign.Start,
        )
        Spacer(modifier = Modifier.width(8.dp))
        Slider(
            value = sliderPosition,
            onValueChange = {
                isUserInteracting = true
                sliderPosition = it
            },
            onValueChangeFinished = {
                isUserInteracting = false
                onSeekTo(sliderPosition)
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
            modifier = modifier.weight(1f)
        )
        Spacer(modifier = Modifier.width(8.dp))
        OutlinedText(
            text = length,
            color = Color.Gray,
            modifier = modifier.width(fixedWidth*fileSize.length),
            textModifier = Modifier.fillMaxWidth(),
            fontSize = fontSize.sp,
            lineHeight = fontSize.sp*1.2f,
            textAlign = TextAlign.End,
        )
    }
}