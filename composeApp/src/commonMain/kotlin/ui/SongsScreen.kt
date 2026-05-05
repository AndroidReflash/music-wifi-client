package ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dataClasses.TrackInfo
import musicwifi.composeapp.generated.resources.Res
import musicwifi.composeapp.generated.resources.pause
import musicwifi.composeapp.generated.resources.player_play_triangle_orange
import musicwifi.composeapp.generated.resources.repeat
import musicwifi.composeapp.generated.resources.shuffle
import musicwifi.composeapp.generated.resources.skip_back
import musicwifi.composeapp.generated.resources.skip_forward
import org.jetbrains.compose.resources.painterResource
import org.koin.compose.viewmodel.koinViewModel
import ui.songScreenElements.AudioVisualizer
import ui.songScreenElements.AudioVisualizerNoBounds
import ui.songScreenElements.CustomSliderSong
import ui.songScreenElements.MarqueeTrackTextBounce
import ui.songScreenElements.MarqueeTrackTextFromRightToLeft
import ui.songScreenElements.SongRow
import viewModel.SongsScreenViewModel

@Composable
fun SongsScreen(
    viewModel: SongsScreenViewModel = koinViewModel()
){
    val songList by viewModel.songList.collectAsState()
    val isPlaying by viewModel.isPlaying.collectAsState()
    val songPlaying by viewModel.currentPlay.collectAsState()
    val equalizer by viewModel.equalizerValues.collectAsState()
    val trackTime by viewModel.trackTimeLabel.collectAsState()
    val trackSize by viewModel.trackSizeLabel.collectAsState()
    val trackProgress by viewModel.trackProgress.collectAsState()
    val shuffle by viewModel.shuffle.collectAsState()
    val repeat by viewModel.repeat.collectAsState()
    val error by viewModel.error.collectAsState()
    val serverIsAlive by viewModel.serverIsAlive.collectAsState()

    LaunchedEffect(Unit){
        viewModel.initSongService()
    }
    Box(Modifier.fillMaxSize().background(Color.Black), contentAlignment = Alignment.Center){
        Box(Modifier.fillMaxWidth()
            .fillMaxHeight(),
            contentAlignment = Alignment.Center
        ){
            AudioVisualizer(equalizer, Modifier.fillMaxWidth(), Color.Red.copy(alpha = 0.3f))
        }

        Column (modifier = Modifier.fillMaxSize().safeDrawingPadding(),
            horizontalAlignment = Alignment.CenterHorizontally
        ) {
            if(error != "" && songList == emptyList<TrackInfo>()){
                Box(Modifier.weight(1f), contentAlignment = Alignment.Center){
                    Text(error, color = Color.White)
                }
            }else{
                LazyColumn(
                    modifier = Modifier.weight(1f).padding(top = 1.dp),
                    verticalArrangement = Arrangement.spacedBy(3.dp)
                ) {
                    items(
                        items = songList,
                        key = { song -> song.trackUrl }
                    ) { song ->
                        val isThisCurrent = song == songPlaying
                        SongRow(song, isThisCurrent){
                            viewModel.play(song)
                        }
                    }
                }
            }
            Box(Modifier.fillMaxWidth()
                .defaultMinSize(minHeight = 30.dp), contentAlignment = Alignment.Center){
                MarqueeTrackTextBounce(
                    songPlaying?.trackName ?: "", Modifier.fillMaxWidth(), 15.sp
                )
            }
            Row(modifier = Modifier.fillMaxWidth()
                .defaultMinSize(minHeight = 60.dp),
                horizontalArrangement = Arrangement.SpaceAround,
                verticalAlignment = Alignment.CenterVertically
            ){
                Icon(
                    painter = painterResource(Res.drawable.shuffle),
                    contentDescription = "Перемешать",
                    modifier = Modifier.size(20.dp)
                        .clickable{
                            viewModel.shuffle()
                        },
                    tint = if(shuffle) Color.Red else Color.White
                )
                Icon(
                    painter = painterResource(Res.drawable.skip_back),
                    contentDescription = "Предыдущий",
                    modifier = Modifier.size(25.dp)
                        .clickable{viewModel.previousTrack() },
                    tint = Color.White
                )
                Icon(
                    painter = painterResource(
                        if(isPlaying) Res.drawable.pause
                        else Res.drawable.player_play_triangle_orange
                    ),
                    contentDescription = "Воспроизведение",
                    modifier = Modifier.size(40.dp)
                        .clickable{if(serverIsAlive) viewModel.play(songPlaying ?: songList[0]) },
                    tint = Color.White
                )
                Icon(
                    painter = painterResource(Res.drawable.skip_forward),
                    contentDescription = "Следующий",
                    modifier = Modifier.size(25.dp)
                        .clickable{viewModel.nextTrack() },
                    tint = Color.White
                )
                Icon(
                    painter = painterResource(Res.drawable.repeat),
                    contentDescription = "Повторить",
                    modifier = Modifier.size(20.dp)
                        .clickable {
                            viewModel.repeat()
                        },
                    tint = if(repeat) Color.Red else Color.White
                )
            }
            CustomSliderSong(
                Modifier,
                trackSize,
                trackTime,
                trackProgress,
                {
                    viewModel.seekTo(it)
                }
            )
        }
    }
}