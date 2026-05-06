package music.repository

import android.content.Context
import dataClasses.TrackInfo
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import repository.interfaces.OfflineRepository
import repository.interfaces.SettingsRepository
import java.io.File

class OfflineRepositoryImpl(
    private val appContext: Context,
    private val scope: CoroutineScope,
    private val settings: SettingsRepository,
): OfflineRepository {

    private val musicFolderUri: StateFlow<String> = settings.folderAddress.stateIn(
        scope = scope,
        started = SharingStarted.WhileSubscribed(5000),
        initialValue = ""
    )

    override suspend fun offlineSongList(): List<TrackInfo> {
        val list = mutableListOf<TrackInfo>()
        try {
            val folderPath = musicFolderUri.value
            val folder = File(folderPath)

            if (!folder.exists() || !folder.isDirectory) return emptyList()

            val files = folder.listFiles { file ->
                file.isFile && file.name.endsWith(".mp3", ignoreCase = true)
            } ?: return emptyList()

            files.forEachIndexed { index, file ->
                list.add(
                    TrackInfo(
                        trackUrl = file.absolutePath,
                        trackName = file.nameWithoutExtension,
                        trackIndex = index,
                        isLocal = true
                    )
                )
            }
            return list
        } catch (e: Exception) {
            return emptyList()
        }
    }

    init {
        scope.launch {
            musicFolderUri.collect {
                println("uri folder $it")
            }
        }
    }
}