package repository.interfaces

import kotlinx.coroutines.flow.StateFlow

interface SettingsRepository {
    val serverAddress: StateFlow<String>
    val isPlayingLocal: StateFlow<Boolean>
    val folderAddress: StateFlow<String>
    suspend fun setServerAddress(address: String)
    suspend fun setOnlineToggle(value: Boolean)
    suspend fun setFolderAddress(address: String)
}