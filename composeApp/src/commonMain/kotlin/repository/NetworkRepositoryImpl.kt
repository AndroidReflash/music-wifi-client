package repository

import com.google.gson.Gson
import dataClasses.ServerConfigDTO
import dataClasses.TrackInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import repository.interfaces.NetworkRepository
import java.io.IOException

class NetworkRepositoryImpl(
    private val client: OkHttpClient,
    private val gson: Gson
): NetworkRepository {

    private val _address = MutableStateFlow("http://192.168.100.9:9090")
    override val address: StateFlow<String> = _address
    override fun setAddress(value: String){
        _address.value = value
    }
    private val _serverUp = MutableStateFlow(false)
    override val serverUp : StateFlow<Boolean> = _serverUp
    private val _fetchError = MutableStateFlow("")
    override val fetchError : StateFlow<String> = _fetchError

    private val _serverConfig = MutableStateFlow(ServerConfigDTO())
    override val serverConfig : StateFlow<ServerConfigDTO> = _serverConfig

    override suspend fun fetchPlaylist(): List<TrackInfo> {
        val url = "${address.value}${serverConfig.value.playlistPath}/${serverConfig.value.playlistName}"
        try {
            val request = Request.Builder()
                .url(url)
                .build()
            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) throw IOException("Ошибка сервера: ${response.code}")
                val text = response.body?.string() ?: ""
                val list = emptyList<TrackInfo>().toMutableList()
                text.lines().filter { it.isNotBlank() && !it.startsWith("#") }.forEachIndexed { index , song ->
                    list += TrackInfo(song, parseName(song), index)
                }
                _fetchError.value = ""
                return list
            }
        }catch (e: Exception){
            e.printStackTrace()
            _fetchError.value = e.message.toString()
        }
        return emptyList()
    }

    override suspend fun getServerConfig() {
        val endPoint = "/config"
        val url = "${address.value}$endPoint"
        val request = Request.Builder().url(url).build()
        try {
            client.newCall(request).execute().use { response ->
                if (response.isSuccessful){
                    val json = response.body?.string()
                    _serverConfig.value = gson.fromJson(json, ServerConfigDTO::class.java)
                    _fetchError.value = ""
                }
            }
        } catch (e: Exception) {
            e.printStackTrace()
            _fetchError.value = e.message.toString()
        }
    }

    override suspend fun getServerStatus() = withContext(Dispatchers.IO){
        val endPoint = "/health"
        val url = "${address.value}$endPoint"

        try {
            val request = Request.Builder().url(url).build()
            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) throw IOException("Ошибка сервера: ${response.code}")
                val text = response.body?.string() ?: ""
                _fetchError.value = text
                _serverUp.value = true
                println("text $text")
            }
        }catch (e: Exception){
            println("exception ${e.message}")
            _fetchError.value = e.message.toString()
            _serverUp.value = false
        }
    }

    private fun parseName(url: String): String{
        return url
            .substringAfter(address.value)
            .substringAfter("${serverConfig.value.musicPath}/")
            .substringBefore(".mp3")

    }
}