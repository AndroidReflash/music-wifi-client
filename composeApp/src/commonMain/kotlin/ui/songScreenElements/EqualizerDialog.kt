package ui.songScreenElements

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.gestures.drag
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import constants.DialogConstantsDp
import kotlin.math.roundToInt

/**
 * Окно эквалайзера: по вертикальному ползунку на полосу, над ним — усиление в дБ, под ним —
 * частота полосы, внизу — сброс всех полос на ноль.
 *
 * [bands] — центры полос, Гц, [gains] — их усиления, дБ, в том же порядке и той же длины.
 * Ползунок ходит от `-maxDb` до `+maxDb`.
 *
 * Как у громкости, звук меняется на отпускании ползунка, а не по ходу движения: значение уходит в
 * настройки, а писать их на каждый сдвиг пальца значило бы писать файл десятки раз за движение.
 */
@Composable
fun EqualizerDialog(
    color: Color,
    bands: List<Float>,
    gains: List<Float>,
    maxDb: Float,
    onBand: (Int, Float) -> Unit,
    onReset: () -> Unit,
    onDismiss: () -> Unit,
) {
    Dialog(
        onDismissRequest = { onDismiss() },
        properties = DialogProperties(usePlatformDefaultWidth = false),
    ) {
        Column(
            Modifier
                .width(DialogConstantsDp.Width.value)
                .background(Color.Black)
                .border(1.dp, Color.White)
                .padding(vertical = 12.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text("Эквалайзер", color = Color.White, style = MaterialTheme.typography.bodyMedium)
            Row(
                Modifier.fillMaxWidth().padding(horizontal = 6.dp),
                horizontalArrangement = Arrangement.SpaceEvenly,
            ) {
                bands.forEachIndexed { band, frequency ->
                    BandSlider(
                        color = color,
                        frequency = frequency,
                        gain = gains[band],
                        maxDb = maxDb,
                        onGain = { gain -> onBand(band, gain) },
                    )
                }
            }
            Text(
                "Сбросить",
                color = Color.White,
                style = MaterialTheme.typography.bodyMedium,
                modifier = Modifier
                    .border(1.dp, Color.White)
                    .clickable { onReset() }
                    .padding(horizontal = 12.dp, vertical = 4.dp),
            )
        }
    }
}

/**
 * Одна полоса: усиление сверху, ползунок, частота снизу.
 *
 * Пока палец тянет, показывается то, что под пальцем, а не записанное: записанное догонит только
 * после отпускания. Своё значение забывается, когда записанное сменилось, — то есть когда
 * отпущенное дошло до настроек или полосы сбросили, — и ползунок не прыгает назад в промежутке.
 */
@Composable
private fun BandSlider(
    color: Color,
    frequency: Float,
    gain: Float,
    maxDb: Float,
    onGain: (Float) -> Unit,
) {
    var shown by remember(gain) { mutableFloatStateOf(gain) }

    Column(horizontalAlignment = Alignment.CenterHorizontally) {
        Text(decibelLabel(shown), color = Color.White, fontSize = 10.sp)
        Canvas(
            Modifier
                .width(28.dp)
                .height(200.dp)
                .pointerInput(gain, maxDb) {
                    // Доля высоты → децибелы: верх — `+maxDb`, низ — `-maxDb`. Шаг — целый
                    // децибел: мельче на слух не различить, а подпись была бы дробной.
                    fun gainAt(y: Float): Float {
                        val share = (y / size.height).coerceIn(0f, 1f)
                        return (maxDb - share * 2 * maxDb).roundToInt().toFloat()
                    }
                    awaitEachGesture {
                        val down = awaitFirstDown()
                        shown = gainAt(down.position.y)
                        drag(down.id) { change ->
                            shown = gainAt(change.position.y)
                            change.consume()
                        }
                        onGain(shown)
                    }
                },
        ) {
            val x = size.width / 2
            val middle = size.height / 2
            val thumb = size.height * (maxDb - shown) / (2 * maxDb)
            val line = 2.dp.toPx()

            drawLine(
                color = Color.Gray.copy(alpha = 0.5f),
                start = Offset(x, 0f),
                end = Offset(x, size.height),
                strokeWidth = line,
            )
            // Отметка нуля: от неё видно, поднята полоса или опущена.
            drawLine(
                color = Color.Gray,
                start = Offset(x - 6.dp.toPx(), middle),
                end = Offset(x + 6.dp.toPx(), middle),
                strokeWidth = line,
            )
            drawLine(
                color = color,
                start = Offset(x, middle),
                end = Offset(x, thumb),
                strokeWidth = line,
                cap = StrokeCap.Round,
            )
            drawCircle(color = color, radius = 6.dp.toPx(), center = Offset(x, thumb))
        }
        Text(frequencyLabel(frequency), color = Color.White, fontSize = 10.sp)
    }
}

/** «+3», «−12», «0»: знак у подъёма тоже, иначе +3 и −3 различались бы только одним. */
private fun decibelLabel(gain: Float): String {
    val whole = gain.roundToInt()
    return when {
        whole > 0 -> "+$whole"
        whole < 0 -> "−${-whole}"
        else -> "0"
    }
}

/** «31», «500», «1к», «16к»: полное число герц в подпись под узким ползунком не влезает. */
private fun frequencyLabel(frequency: Float): String =
    if (frequency >= 1000f) "${(frequency / 1000f).roundToInt()}к" else "${frequency.toInt()}"
