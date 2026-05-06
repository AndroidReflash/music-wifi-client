package repository


import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import repository.interfaces.SettingsRepository

class SettingsRepositoryImpl(
    private val dataStore: DataStore<Preferences>,
    private val scope: CoroutineScope
): SettingsRepository {

    private object Keys {
        val SERVER_ADDRESS = stringPreferencesKey("server_address")
        val ONLINE_TOGGLE = booleanPreferencesKey("online_toggle")
        val OFFLINE_FOLDER_ADDRESS = stringPreferencesKey("folder_address")
    }

    override val serverAddress: StateFlow<String> = dataStore.data
        .map { preferences ->
            preferences[Keys.SERVER_ADDRESS] ?: ""
        }.stateIn(
            scope = scope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = ""
        )

    override val folderAddress: StateFlow<String> = dataStore.data
        .map { preferences ->
            preferences[Keys.OFFLINE_FOLDER_ADDRESS] ?: ""
        }.stateIn(
            scope = scope,
            started = SharingStarted.WhileSubscribed(5000),
            initialValue = ""
        )

    override val isPlayingLocal: StateFlow<Boolean> = dataStore.data
        .map { it[Keys.ONLINE_TOGGLE] ?: false }
        .stateIn(scope, SharingStarted.WhileSubscribed(5000), false)

    override suspend fun setServerAddress(address: String) {
        dataStore.edit { preferences ->
            preferences[Keys.SERVER_ADDRESS] = address
        }
    }

    override suspend fun setOnlineToggle(value: Boolean){
        dataStore.edit { preferences ->
            preferences[Keys.ONLINE_TOGGLE] = value
        }
    }

    override suspend fun setFolderAddress(address: String) {
        dataStore.edit { preferences ->
            preferences[Keys.OFFLINE_FOLDER_ADDRESS] = address
        }
    }
}