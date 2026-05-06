package ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import musicwifi.composeapp.generated.resources.Res
import musicwifi.composeapp.generated.resources.menu
import musicwifi.composeapp.generated.resources.pause
import musicwifi.composeapp.generated.resources.player_play_triangle_orange
import musicwifi.composeapp.generated.resources.repeat
import musicwifi.composeapp.generated.resources.shuffle
import musicwifi.composeapp.generated.resources.skip_back
import musicwifi.composeapp.generated.resources.skip_forward
import org.jetbrains.compose.resources.painterResource
import org.koin.compose.viewmodel.koinViewModel
import ui.songScreenElements.AudioVisualizerHeight
import ui.songScreenElements.ChangeIpDialog
import ui.songScreenElements.CustomSliderSong
import ui.songScreenElements.MarqueeTrackTextBounce
import ui.songScreenElements.SongRow
import viewModel.SongsScreenViewModel

@Composable
fun SongsScreen(
    interfaceColor: Color,
    viewModel: SongsScreenViewModel = koinViewModel()
){
    val songListOffline by viewModel.songListOffline.collectAsState()
    val songListOnline by viewModel.songListOnline.collectAsState()
    val isPlaying by viewModel.isPlaying.collectAsState()
    val songPlaying by viewModel.currentPlay.collectAsState()
    val equalizer by viewModel.equalizerValues.collectAsState()
    val trackTime by viewModel.trackTimeLabel.collectAsState()
    val trackSize by viewModel.trackSizeLabel.collectAsState()
    val trackProgress by viewModel.trackProgress.collectAsState()
    val shuffle by viewModel.shuffle.collectAsState()
    val repeat by viewModel.repeat.collectAsState()
    val error by viewModel.error.collectAsState()
    val isServerAvailable by viewModel.isServerAvailable.collectAsState()
    val currentIp by viewModel.currentIp.collectAsState()
    val isLocal by viewModel.isLocal.collectAsState()
    val folderAddress by viewModel.folderAddress.collectAsState()

    var serverIp by remember { mutableStateOf("") }
    var showPopUp by remember { mutableStateOf(false) }

    Box(Modifier.fillMaxSize().background(Color.Black), contentAlignment = Alignment.Center){
        Box(Modifier.fillMaxWidth()
            .fillMaxHeight(),
            contentAlignment = Alignment.Center
        ){
            AudioVisualizerHeight(equalizer, Modifier.fillMaxWidth(), interfaceColor.copy(alpha = 0.3f))
        }

        Column (modifier = Modifier.fillMaxSize().systemBarsPadding(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            Row(Modifier.fillMaxWidth().height(30.dp).padding(start = 5.dp, end = 5.dp),
                horizontalArrangement = Arrangement.SpaceBetween
            ){
                Row(
                    Modifier.weight(1f),
                    horizontalArrangement = Arrangement.spacedBy(5.dp),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    if(!isLocal){
                        Box(Modifier.size(15.dp)
                            .clip(RoundedCornerShape(7.5.dp))
                            .background(if(isServerAvailable) interfaceColor else Color.White)
                        )
                        Text(if(isServerAvailable) "В сети" else "Недоступен", color = Color.White)
                    }else{
                        Text(" ", color = Color.White)
                    }
                }
                Icon(
                    painter = painterResource(Res.drawable.menu),
                    contentDescription = "Всплывашка",
                    modifier = Modifier.size(20.dp).weight(1f)
                        .clickable{
                            showPopUp = true
                        },
                    tint = Color.White
                )
                if(songPlaying?.trackIndex != null){
                    if(isLocal){
                        Text("${songPlaying!!.trackIndex + 1} / ${songListOffline.size}",
                            color = Color.White,
                            modifier = Modifier.weight(1f),
                            textAlign = TextAlign.End
                        )
                    }else{
                        Text("${songPlaying!!.trackIndex + 1} / ${songListOnline.size}",
                            color = Color.White,
                            modifier = Modifier.weight(1f),
                            textAlign = TextAlign.End
                        )
                    }
                }else{
                    if(isLocal){
                        Text("${songListOffline.size}",
                            color = Color.White,
                            modifier = Modifier.weight(1f),
                            textAlign = TextAlign.End
                        )
                    }else{
                        Text("${songListOnline.size}",
                            color = Color.White,
                            modifier = Modifier.weight(1f),
                            textAlign = TextAlign.End
                        )
                    }
                }
            }
            if(!isServerAvailable && !isLocal){
                Box(Modifier.weight(1f), contentAlignment = Alignment.Center){
                    Text(error, color = Color.White, textAlign = TextAlign.Center)
                }
            }else{
                LazyColumn(
                    modifier = Modifier.weight(1f).padding(top = 1.dp),
                    verticalArrangement = Arrangement.spacedBy(3.dp)
                ) {
                    items(
                        items = if(isLocal) songListOffline else songListOnline,
                        key = { song -> song.trackUrl }
                    ) { song ->
                        val isThisCurrent = song == songPlaying
                        SongRow(interfaceColor, song, isThisCurrent){
                            viewModel.play(song)
                            if(shuffle && !isThisCurrent){
                                viewModel.onSelectedManually(song)
                            }
                        }
                    }
                }
            }
            Box(Modifier.fillMaxWidth()
                .height(30.dp), contentAlignment = Alignment.Center){
                MarqueeTrackTextBounce(
                    songPlaying?.trackName ?: "", Modifier.fillMaxWidth(), 15.sp
                )
            }
            Row(modifier = Modifier.fillMaxWidth()
                .height(60.dp),
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
                    tint = if(shuffle) interfaceColor else Color.White
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
                        .clickable{
                            if(songPlaying != null){
                                viewModel.play(songPlaying!!)
                            }else{
                                val currentList = if (isLocal) songListOffline else songListOnline
                                if(currentList.isNotEmpty()){
                                    val number = 0..<currentList.size
                                    val currentTrack = currentList[number.random()]
                                    viewModel.play(currentTrack)
                                    if(shuffle){
                                        viewModel.onSelectedManually(currentTrack)
                                    }
                                }
                            }
                        },
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
                    tint = if(repeat) interfaceColor else Color.White
                )
            }
            CustomSliderSong(
                interfaceColor,
                Modifier,
                trackSize,
                trackTime,
                trackProgress,
                {
                    viewModel.seekTo(it)
                }
            )
        }
        if(showPopUp){
            ChangeIpDialog(
                color = interfaceColor,
                isLocal = isLocal,
                onSwitch = {
                    viewModel.isLocal()
                },
                onDismiss = {
                    showPopUp = false
                },
                onConfirmOnline = {
                    serverIp = it
                    viewModel.setServerIp(serverIp)
                    showPopUp = false
                },
                onChooseOfflineFolder = {
                    viewModel.onSelectFolderClicked()
                },
                currentIp = currentIp,
                folderAddress =  folderAddress
            )
        }
    }
}