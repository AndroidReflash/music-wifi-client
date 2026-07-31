package repository

import dataClasses.ServerConfigDTO
import dataClasses.TrackInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flowOn
import kotlinx.coroutines.launch
import repository.interfaces.NetworkRepository
import repository.interfaces.SettingsRepository
import sealed.Keywords
import uniffi.player_core.Command
import uniffi.player_core.QuicSession
import uniffi.player_core.SseCallback

class NetworkRepositoryImplRust(
    private val settings: SettingsRepository,
) : NetworkRepository {

    private val _isServerAvailable = MutableStateFlow(false)
    override val isServerAvailable: StateFlow<Boolean> = _isServerAvailable

    override val address: Flow<String> = settings.serverAddress

    private val _fetchError = MutableStateFlow("")
    override val fetchError: StateFlow<String> = _fetchError

    private val _serverConfig = MutableStateFlow(ServerConfigDTO())
    override val serverConfig: StateFlow<ServerConfigDTO> = _serverConfig

    private var session: QuicSession? = null
    private var connectedAddress: String = ""

    private suspend fun getOrCreateSession(): QuicSession? {
        val currentAddress = settings.serverAddress.first { it.isNotBlank() }
        if (session == null || connectedAddress != currentAddress) {
            try {
                session = null
                val newSession = QuicSession.connect(currentAddress)
                session = newSession
                connectedAddress = currentAddress
                _isServerAvailable.value = true
                _fetchError.value = ""
            } catch (e: Exception) {
                e.printStackTrace()
                _fetchError.value = e.message ?: "Failed to connect to QUIC server"
                _isServerAvailable.value = false
                session = null
                connectedAddress = ""
                return null
            }
        }
        return session
    }

    override suspend fun fetchPlaylist(): List<TrackInfo> {
        val currentSession = getOrCreateSession() ?: return emptyList()
        return try {
            val responseBytes = currentSession.sendCommand(Command.GET_PLAYLIST, byteArrayOf())
            val text = responseBytes.decodeToString()
            
            val list = mutableListOf<TrackInfo>()
            text.lines()
                .filter { it.isNotBlank() && !it.startsWith("#") }
                .forEachIndexed { index, song ->
                    // Путь файла на сервере, как его ждёт QUIC: плеер читает дорожку
                    // прямо оттуда, локальный прокси ему не нужен.
                    val songPath = song.removePrefix("/")
                    list.add(
                        TrackInfo(
                            trackUrl = songPath,
                            trackName = song.substringAfterLast("/").substringBeforeLast("."),
                            trackIndex = index,
                            isLocal = false
                        )
                    )
                }
            _fetchError.value = ""
            _isServerAvailable.value = true
            list
        } catch (e: Exception) {
            e.printStackTrace()
            _fetchError.value = e.message ?: "Failed to fetch playlist over QUIC"
            _isServerAvailable.value = false
            session = null
            connectedAddress = ""
            emptyList()
        }
    }

    override suspend fun getServerConfig() {
        _fetchError.value = ""
    }

    override fun observeServerUpdates(): Flow<String> = callbackFlow {
        val job = launch(Dispatchers.IO) {
            var subJob: kotlinx.coroutines.Job? = null
            settings.serverAddress.collect { address ->
                if (address.isBlank()) return@collect
                subJob?.cancel()
                session = null
                connectedAddress = ""
                val currentSession = getOrCreateSession()
                if (currentSession != null) {
                    _isServerAvailable.value = true
                    trySend(Keywords.Refresh.value)
                    subJob = launch(Dispatchers.IO) {
                        try {
                            currentSession.subscribeSse(object : SseCallback {
                                override fun onEvent(message: String) {
                                    _isServerAvailable.value = true
                                    trySend(Keywords.Refresh.value)
                                }
                            })
                        } catch (e: Exception) {
                            e.printStackTrace()
                            _isServerAvailable.value = false
                            session = null
                            connectedAddress = ""
                        }
                    }
                } else {
                    _isServerAvailable.value = false
                }
            }
        }
        awaitClose {
            job.cancel()
            _isServerAvailable.value = false
        }
    }.flowOn(Dispatchers.IO)
}
