package repository

import com.google.gson.Gson
import dataClasses.ServerConfigDTO
import dataClasses.TrackInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.flow.retry
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.Response
import okhttp3.sse.EventSource
import okhttp3.sse.EventSourceListener
import okhttp3.sse.EventSources
import repository.interfaces.NetworkRepository
import repository.interfaces.SettingsRepository
import sealed.Keywords
import java.io.IOException

class NetworkRepositoryImpl(
    private val settings: SettingsRepository,
    private val client: OkHttpClient,
    private val gson: Gson,
): NetworkRepository {
    private val _isServerAvailable = MutableStateFlow(false)
    override val isServerAvailable : StateFlow<Boolean> = _isServerAvailable
    override val address: StateFlow<String> = settings.serverAddress
    private val _fetchError = MutableStateFlow("")
    override val fetchError : StateFlow<String> = _fetchError

    private val _serverConfig = MutableStateFlow(ServerConfigDTO())
    override val serverConfig : StateFlow<ServerConfigDTO> = _serverConfig

    override suspend fun fetchPlaylist(): List<TrackInfo> {
        val baseUrl = checkCorrectUrl()
        val url = "$baseUrl${serverConfig.value.playlistPath}/${serverConfig.value.playlistName}"
        try {
            val request = Request.Builder()
                .url(url)
                .build()
            client.newCall(request).execute().use { response ->
                if (!response.isSuccessful) throw IOException("Ошибка сервера: ${response.code}")
                val text = response.body?.string() ?: ""
                val list = emptyList<TrackInfo>().toMutableList()
                text.lines().filter { it.isNotBlank() && !it.startsWith("#") }.forEachIndexed { index , song ->
                    val songUrl = "$baseUrl${song}"
                    println(songUrl)
                    list += TrackInfo(songUrl, parseName(song), index, true)
                }
                _fetchError.value = ""
                return list
            }
        }catch (e: Exception){
            e.printStackTrace()
            _fetchError.value = e.message.toString()
            _isServerAvailable.value = false
        }
        return emptyList()
    }

    override suspend fun getServerConfig() {
        val sseUrl = "${checkCorrectUrl()}${Keywords.ConfigEndpoint.value}"
        val request = Request.Builder().url(sseUrl).build()
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
            _isServerAvailable.value = false
        }
    }

    override fun observeServerUpdates(): Flow<String> = callbackFlow {
        val sseUrl = "${checkCorrectUrl()}${Keywords.SSESub.value}"
        val request = Request.Builder()
            .url(sseUrl)
            .build()

        val listener = object : EventSourceListener() {
            override fun onOpen(eventSource: EventSource, response: Response) {
                _fetchError.value = ""
                _isServerAvailable.value = true
            }

            override fun onEvent(eventSource: EventSource, id: String?, type: String?, data: String) {
                _isServerAvailable.value = true
                trySend(Keywords.Refresh.value)
            }

            override fun onFailure(eventSource: EventSource, t: Throwable?, response: Response?) {
                _isServerAvailable.value = false
                _fetchError.value = t?.message ?: "Connection lost"
                close(t ?: IOException("SSE connection failed"))
            }
        }
        val eventSource = EventSources.createFactory(client).newEventSource(request, listener)
        awaitClose {
            eventSource.cancel()
            _isServerAvailable.value = false
        }
    }.retry { cause ->
        val shouldRetry = cause is IOException
        if (shouldRetry) delay(5000)
        shouldRetry
    }.flowOn(Dispatchers.IO)

    private fun parseName(url: String): String{
        return url
            .substringAfter("${serverConfig.value.musicPath}/")
            .substringBefore(".mp3")

    }

    private suspend fun checkCorrectUrl(): String{
        val rawAddress = address.first { it.isNotBlank() }
        println("address ${address.first { it.isNotBlank() }}")
        val baseUrl = if (rawAddress.startsWith("http://") || rawAddress.startsWith("https://")) {
            rawAddress
        } else {
            "http://$rawAddress"
        }

        return baseUrl
    }
}