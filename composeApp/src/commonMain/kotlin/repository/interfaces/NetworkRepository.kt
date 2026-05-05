package repository.interfaces


import dataClasses.ServerConfigDTO
import dataClasses.TrackInfo
import kotlinx.coroutines.flow.StateFlow

interface NetworkRepository {
    suspend fun fetchPlaylist(): List<TrackInfo>
    suspend fun getServerConfig()
    fun setAddress(value: String)
    suspend fun getServerStatus()

    val serverConfig : StateFlow<ServerConfigDTO>
    val fetchError : StateFlow<String>
    val address: StateFlow<String>
    val serverUp : StateFlow<Boolean>
}