package ui.songScreenElements

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import dataClasses.TrackInfo

/**
 * Список дорожек — или беда вместо него: когда играть нечего и сказать об этом надо, на месте
 * списка стоит текст. Место у них одно, поэтому и блок один.
 *
 * [error] непустая означает «списка нет, показывай это». [songs] приходят уже отобранными:
 * отбор по запросу поиска нужен не только здесь, а значит считается снаружи.
 */
@Composable
fun SongsList(
    songs: List<TrackInfo>,
    songPlaying: TrackInfo?,
    interfaceColor: Color,
    textColor: Color,
    listState: LazyListState,
    shuffle: Boolean,
    error: String?,
    onPlay: (TrackInfo) -> Unit,
    onSelectedManually: (TrackInfo) -> Unit,
    modifier: Modifier = Modifier,
) {
    if (error != null) {
        Box(modifier, contentAlignment = Alignment.Center) {
            Text(error, color = textColor, textAlign = TextAlign.Center)
        }
        return
    }

    LazyColumn(
        state = listState,
        modifier = modifier.padding(top = 1.dp),
        verticalArrangement = Arrangement.spacedBy(3.dp)
    ) {
        items(
            items = songs,
            key = { song -> song.trackUrl }
        ) { song ->
            val isThisCurrent = song == songPlaying
            SongRow(interfaceColor, song, isThisCurrent) {
                onPlay(song)
                if (shuffle && !isThisCurrent) {
                    onSelectedManually(song)
                }
            }
        }
    }
}
