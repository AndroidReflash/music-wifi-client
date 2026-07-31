package repository

import dataClasses.TrackInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import repository.interfaces.OfflineRepository
import repository.interfaces.SettingsRepository
import uniffi.player_core.FolderWatcher
import uniffi.player_core.FolderWatcherCallback
import uniffi.player_core.TrackData

class OfflineRepositoryImplRust(
    private val scope: CoroutineScope,
    private val settings: SettingsRepository,
): OfflineRepository {
    private val _offlineSongs = MutableStateFlow(emptyList<TrackInfo>())
    override val offlineSongsFlow: StateFlow<List<TrackInfo>> = _offlineSongs

    private var watcher: FolderWatcher? = null
    private var currentFolderPath: String = ""


    init {
        scope.launch {
            settings.folderAddress
                .onStart { emit(settings.folderAddress.value) }
                .collect { folderPath ->
                    startWatcher(folderPath)
                }
        }
    }

    private fun startWatcher(folderPath: String) {
        if (folderPath == currentFolderPath && watcher != null) return

        watcher?.stop()
        watcher = null
        currentFolderPath = ""

        if (folderPath.isBlank()) {
            _offlineSongs.value = emptyList()
            return
        }

        currentFolderPath = folderPath
        val callback = object : FolderWatcherCallback {
            override fun onChanged(tracks: List<TrackData>) {
                val mapped = tracks.map { t ->
                    TrackInfo(
                        trackUrl = t.trackUrl,
                        trackName = t.trackName,
                        trackIndex = t.trackIndex,
                        isLocal = t.isLocal
                    )
                }
                _offlineSongs.value = mapped
            }
        }

        watcher = FolderWatcher(folderPath, callback)
    }
}