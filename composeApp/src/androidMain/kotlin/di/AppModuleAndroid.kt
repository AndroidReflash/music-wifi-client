package di

import androidx.datastore.core.DataStore
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import music.repository.SongsRepositoryImpl
import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import repository.interfaces.SongsRepository
import androidx.datastore.preferences.core.PreferenceDataStoreFactory
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.preferencesDataStoreFile
import dispatcher.FolderPickerDispatcherImpl
import dispatchers.FolderPickerDispatcher
import music.repository.OfflineRepositoryImpl
import org.koin.android.ext.koin.androidContext
import repository.interfaces.OfflineRepository

val appModuleAndroid = module {
    includes(appModule)

    single<DataStore<Preferences>> {
        PreferenceDataStoreFactory.create(
            produceFile = {
                androidContext().preferencesDataStoreFile("settings")
            }
        )
    }

    single { CoroutineScope(SupervisorJob() + Dispatchers.Default) }
    singleOf(::SongsRepositoryImpl) bind SongsRepository::class
    singleOf(::OfflineRepositoryImpl) bind OfflineRepository::class
    singleOf(::FolderPickerDispatcherImpl) bind FolderPickerDispatcher::class
}
