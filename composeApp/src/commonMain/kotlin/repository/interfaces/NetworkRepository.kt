package repository.interfaces


import dataClasses.ServerConfigDTO
import dataClasses.TrackInfo
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

interface NetworkRepository {
    suspend fun fetchPlaylist(): List<TrackInfo>
    suspend fun getServerConfig()
    fun observeServerUpdates() : Flow<String>

    val serverConfig : StateFlow<ServerConfigDTO>
    val fetchError : StateFlow<String>
    val address: Flow<String>
    val isServerAvailable : StateFlow<Boolean>
}