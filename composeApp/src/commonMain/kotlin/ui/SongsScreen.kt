package ui

import androidx.compose.foundation.gestures.awaitEachGesture
import androidx.compose.foundation.gestures.awaitFirstDown
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.PointerEventPass
import androidx.compose.ui.input.pointer.pointerInput
import kotlinx.coroutines.launch
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel
import platform.interfaces.HotkeySettings
import platform.interfaces.PanelSettings
import platform.interfaces.SongsLayout
import platform.interfaces.SoundSettings
import enums.VisualizerShape
import ui.songScreenElements.AudioVisualizerHeight
import ui.songScreenElements.AudioVisualizerRadial
import ui.songScreenElements.ChangeIpDialog
import ui.songScreenElements.EqualizerDialog
import ui.songScreenElements.CoverArt
import ui.songScreenElements.CoverImage
import ui.songScreenElements.CustomSliderSong
import ui.songScreenElements.SongsControls
import ui.songScreenElements.SongsList
import ui.songScreenElements.SongsTitleBar
import ui.songScreenElements.SongsTopBar
import viewModel.SongsScreenViewModel

/**
 * Экран воспроизведения: собирает состояние и отдаёт блоки той расстановке, которая досталась
 * платформе. Где что стоит, здесь не решается — [SongsLayout].
 */
@Composable
fun SongsScreen(
    textColor: Color,
    backgroundColor: Color,
    // Идёт ли набор текста в поле поиска. Нужно горячим клавишам десктопа: пока курсор в поле,
    // нажатия принадлежат ему. Телефону не нужно, отсюда пустое по умолчанию.
    onTextEntry: (Boolean) -> Unit = {},
    viewModel: SongsScreenViewModel = koinViewModel(),
    layout: SongsLayout = koinInject(),
    soundSettings: SoundSettings = koinInject(),
    panelSettings: PanelSettings = koinInject(),
    hotkeySettings: HotkeySettings = koinInject(),
){
    val songListOffline by viewModel.songListOffline.collectAsState()
    val songListOnline by viewModel.songListOnline.collectAsState()
    val playlist by viewModel.playlist.collectAsState()
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
    val currentInterfaceColor by viewModel.interfaceColor.collectAsState()
    val showVisualizer by viewModel.showVisualizer.collectAsState()
    val showCover by viewModel.showCover.collectAsState()
    val outputDevice by viewModel.outputDevice.collectAsState()
    val outputDevices by viewModel.outputDevices.collectAsState()
    val volume by viewModel.volume.collectAsState()
    val equalizerGains by viewModel.equalizerGains.collectAsState()
    val hotkeys by viewModel.hotkeys.collectAsState()
    val hotkeysGlobal by viewModel.hotkeysGlobal.collectAsState()
    val autoplay by viewModel.autoplay.collectAsState()
    val visualizerRing by viewModel.visualizerRing.collectAsState()
    val coverSmall by viewModel.coverSmall.collectAsState()
    val showCoverImage by viewModel.showCoverImage.collectAsState()
    val coverPicture by viewModel.coverImage.collectAsState()
    val panelDivider by viewModel.panelDivider.collectAsState()
    val panelTransparent by viewModel.panelTransparent.collectAsState()

    var serverIp by remember { mutableStateOf("") }
    var showPopUp by remember { mutableStateOf(false) }
    var showEqualizer by remember { mutableStateOf(false) }
    var isSearchActive by remember { mutableStateOf(false) }
    var searchQuery by remember { mutableStateOf("") }

    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()

    // Отбор по запросу поиска нужен и списку, и прокрутке к играющей дорожке, поэтому
    // считается один раз здесь.
    val displayedSongs = if (searchQuery.isNotBlank()) {
        playlist.filter { it.trackName.contains(searchQuery, ignoreCase = true) }
    } else {
        playlist
    }

    // Спрятанное отдаётся как `null`, а не как пустой кусок разметки: место под него занимать
    // незачем, а убрать его может только расстановка.
    val shape = if (visualizerRing) VisualizerShape.RING else VisualizerShape.BARS
    val visualizerSlot: (@Composable (Modifier, Color) -> Unit)? = if (showVisualizer) {
        { modifier, color ->
            when (shape) {
                VisualizerShape.BARS -> AudioVisualizerHeight(equalizer, modifier, color)
                VisualizerShape.RING -> AudioVisualizerRadial(equalizer, modifier, color)
            }
        }
    } else {
        null
    }
    val coverSlot: (@Composable (Modifier, Color) -> Unit)? = if (showCover) {
        { modifier, color ->
            CoverArt(
                color = color,
                isPlaying = isPlaying,
                small = coverSmall,
                modifier = modifier,
            )
        }
    } else {
        null
    }
    val coverImageSlot: (@Composable (Modifier) -> Unit)? = if (showCoverImage) {
        { modifier -> CoverImage(image = coverPicture, modifier = modifier) }
    } else {
        null
    }

    // Любое нажатие на экране — знак, что пользователь взялся за приложение сам, и автоплей, если
    // ещё ждёт списка, больше не нужен. Нажатие видно здесь раньше, чем кнопке под пальцем
    // (`PointerEventPass.Initial`), и у неё не отнимается. Окно настроек — отдельное окно, его
    // нажатия сюда не доходят, но чтобы его открыть, нажимают кнопку здесь.
    // `propagateMinConstraints` — чтобы расстановка получала тот же размер, что и без этой
    // обёртки: весь экран.
    Box(
        modifier = Modifier
            .fillMaxSize()
            .pointerInput(Unit) {
                awaitEachGesture {
                    awaitFirstDown(requireUnconsumed = false, pass = PointerEventPass.Initial)
                    viewModel.stopAutoplay()
                }
            },
        propagateMinConstraints = true,
    ) {
        layout.Content(
            backgroundColor = backgroundColor,
            interfaceColor = currentInterfaceColor,
            listState = listState,
            visualizer = visualizerSlot,
            shape = shape,
            topBar = { dimensions ->
                SongsTopBar(
                    textColor = textColor,
                    interfaceColor = currentInterfaceColor,
                    isLocal = isLocal,
                    isServerAvailable = isServerAvailable,
                    trackCount = if (isLocal) songListOffline.size else songListOnline.size,
                    currentNumber = songPlaying?.let { it.trackIndex + 1 },
                    isSearchActive = isSearchActive,
                    searchQuery = searchQuery,
                    onSearchActiveChange = { isSearchActive = it },
                    onSearchQueryChange = { searchQuery = it },
                    onMenu = { showPopUp = true },
                    onEqualizer = { showEqualizer = true },
                    onTextEntry = onTextEntry,
                    dimensions = dimensions,
                )
            },
            list = { modifier ->
                SongsList(
                    songs = displayedSongs,
                    songPlaying = songPlaying,
                    interfaceColor = currentInterfaceColor,
                    textColor = textColor,
                    listState = listState,
                    shuffle = shuffle,
                    error = if (!isServerAvailable && !isLocal) error else null,
                    onPlay = { song -> viewModel.play(song) },
                    onSelectedManually = { song -> viewModel.onSelectedManually(song) },
                    modifier = modifier,
                )
            },
            coverImage = coverImageSlot,
            cover = coverSlot,
            title = { modifier ->
                SongsTitleBar(
                    title = songPlaying?.trackName ?: "",
                    onClick = {
                        songPlaying?.let { playingSong ->
                            val index = displayedSongs.indexOf(playingSong)
                            if (index != -1) {
                                scope.launch {
                                    listState.animateScrollToItem(index)
                                }
                            }
                        }
                    },
                    modifier = modifier,
                )
            },
            controls = { modifier ->
                SongsControls(
                    playlist = playlist,
                    songPlaying = songPlaying,
                    isPlaying = isPlaying,
                    shuffle = shuffle,
                    repeat = repeat,
                    textColor = textColor,
                    interfaceColor = currentInterfaceColor,
                    onPlay = { song -> viewModel.play(song) },
                    onSelectedManually = { song -> viewModel.onSelectedManually(song) },
                    onPrevious = { viewModel.previousTrack() },
                    onNext = { viewModel.nextTrack() },
                    onShuffle = { viewModel.shuffle() },
                    onRepeat = { viewModel.repeat() },
                    modifier = modifier,
                )
            },
            panelDivider = panelDivider,
            panelTransparent = panelTransparent,
            slider = { modifier ->
                CustomSliderSong(
                    currentInterfaceColor,
                    modifier,
                    trackSize,
                    trackTime,
                    trackProgress,
                    {
                        viewModel.seekTo(it)
                    }
                )
            },
        )
    }

    if (showEqualizer) {
        EqualizerDialog(
            color = currentInterfaceColor,
            bands = viewModel.equalizerBands,
            gains = equalizerGains,
            maxDb = viewModel.equalizerMaxDb,
            onBand = { band, gain -> viewModel.setEqualizerBand(band, gain) },
            onReset = { viewModel.resetEqualizer() },
            onDismiss = { showEqualizer = false },
        )
    }

    // Диалог — настоящий `Dialog`, своё окно: где он стоит в разметке, на его место не влияет.
    if(showPopUp){
        ChangeIpDialog(
            color = currentInterfaceColor,
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
            folderAddress =  folderAddress,
            onThemeSelected = { newColor ->
                viewModel.setThemeColor(newColor)
            },
            showVisualizer = showVisualizer,
            showCover = showCover,
            showCoverImage = showCoverImage,
            visualizerRing = visualizerRing,
            coverSmall = coverSmall,
            onShowVisualizer = { value -> viewModel.setShowVisualizer(value) },
            onShowCover = { value -> viewModel.setShowCover(value) },
            onShowCoverImage = { value -> viewModel.setShowCoverImage(value) },
            onVisualizerRing = { value -> viewModel.setVisualizerRing(value) },
            onCoverSmall = { value -> viewModel.setCoverSmall(value) },
            outputDevices = outputDevices,
            outputDevice = outputDevice,
            volume = volume,
            onRefreshOutputDevices = { viewModel.refreshOutputDevices() },
            onPickOutputDevice = { name -> viewModel.setOutputDevice(name) },
            onVolume = { value -> viewModel.setVolume(value) },
            panelDivider = panelDivider,
            panelTransparent = panelTransparent,
            onPanelDivider = { value -> viewModel.setPanelDivider(value) },
            onPanelTransparent = { value -> viewModel.setPanelTransparent(value) },
            hotkeys = hotkeys,
            onHotkey = { action, hotkey -> viewModel.setHotkey(action, hotkey) },
            hotkeysGlobal = hotkeysGlobal,
            onHotkeysGlobal = { value -> viewModel.setHotkeysGlobal(value) },
            autoplay = autoplay,
            onAutoplay = { value -> viewModel.setAutoplay(value) },
            soundSettings = soundSettings,
            panelSettings = panelSettings,
            hotkeySettings = hotkeySettings,
        )
    }
}
