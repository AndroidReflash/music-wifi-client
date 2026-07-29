package repository.interfaces

import androidx.compose.ui.graphics.Color
import kotlinx.coroutines.flow.StateFlow

interface SettingsRepository {
    val serverAddress: StateFlow<String>
    val isPlayingLocal: StateFlow<Boolean>
    val folderAddress: StateFlow<String>
    val interfaceColor: StateFlow<Color>
    suspend fun setServerAddress(address: String)
    suspend fun setOnlineToggle(value: Boolean)
    suspend fun setFolderAddress(address: String)
    suspend fun setThemeColor(color: Color)
}
