package ui.songScreenElements

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dataClasses.TrackInfo

@Composable
fun SongRow(song: TrackInfo, isThisCurrent: Boolean, play: () -> Unit){
    Box(
        Modifier.fillMaxWidth()
        .wrapContentHeight().defaultMinSize(minHeight = 60.dp)
        .padding(5.dp)
        .border(
            1.dp,
            if(isThisCurrent) Color.Red else Color.LightGray,
            RoundedCornerShape(5.dp)
        )
        .clickable{
            play()
        },
        contentAlignment = Alignment.Center
    ){
        Text(song.trackName, Modifier.padding(start = 4.dp, end = 4.dp),
            maxLines = 1, color = Color.White, overflow = TextOverflow.Ellipsis
        )
    }
}