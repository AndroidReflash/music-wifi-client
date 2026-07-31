package ui.songScreenElements

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import dataClasses.TrackInfo
import musicwifi.composeapp.generated.resources.Res
import musicwifi.composeapp.generated.resources.pause
import musicwifi.composeapp.generated.resources.player_play_triangle_orange
import musicwifi.composeapp.generated.resources.repeat
import musicwifi.composeapp.generated.resources.shuffle
import musicwifi.composeapp.generated.resources.skip_back
import musicwifi.composeapp.generated.resources.skip_forward
import org.jetbrains.compose.resources.painterResource

/**
 * Кнопки управления: перемешать, назад, играть или пауза, вперёд, повтор.
 *
 * Средняя кнопка на пустом месте — когда не играет ничего — берёт из списка случайную дорожку.
 * Выбор живёт здесь, а не у вызывающего, потому что на обеих платформах он один и тот же.
 */
@Composable
fun SongsControls(
    playlist: List<TrackInfo>,
    songPlaying: TrackInfo?,
    isPlaying: Boolean,
    shuffle: Boolean,
    repeat: Boolean,
    textColor: Color,
    interfaceColor: Color,
    onPlay: (TrackInfo) -> Unit,
    onSelectedManually: (TrackInfo) -> Unit,
    onPrevious: () -> Unit,
    onNext: () -> Unit,
    onShuffle: () -> Unit,
    onRepeat: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier.fillMaxWidth()
            .height(60.dp),
        horizontalArrangement = Arrangement.SpaceAround,
        verticalAlignment = Alignment.CenterVertically
    ) {
        OutlinedIcon(
            painter = painterResource(Res.drawable.shuffle),
            contentDescription = "Перемешать",
            tint = if (shuffle) interfaceColor else textColor,
            modifier = Modifier.size(20.dp)
                .clickable { onShuffle() },
        )
        OutlinedIcon(
            painter = painterResource(Res.drawable.skip_back),
            contentDescription = "Предыдущий",
            tint = textColor,
            modifier = Modifier.size(25.dp)
                .clickable { onPrevious() },
        )
        OutlinedIcon(
            painter = painterResource(
                if (isPlaying) Res.drawable.pause
                else Res.drawable.player_play_triangle_orange
            ),
            contentDescription = "Воспроизведение",
            tint = textColor,
            modifier = Modifier.size(40.dp)
                .clickable {
                    if (songPlaying != null) {
                        onPlay(songPlaying)
                    } else {
                        if (playlist.isNotEmpty()) {
                            val number = 0..<playlist.size
                            val currentTrack = playlist[number.random()]
                            onPlay(currentTrack)
                            if (shuffle) {
                                onSelectedManually(currentTrack)
                            }
                        }
                    }
                },
        )
        OutlinedIcon(
            painter = painterResource(Res.drawable.skip_forward),
            contentDescription = "Следующий",
            tint = textColor,
            modifier = Modifier.size(25.dp)
                .clickable { onNext() },
        )
        OutlinedIcon(
            painter = painterResource(Res.drawable.repeat),
            contentDescription = "Повторить",
            tint = if (repeat) interfaceColor else textColor,
            modifier = Modifier.size(20.dp)
                .clickable { onRepeat() },
        )
    }
}
