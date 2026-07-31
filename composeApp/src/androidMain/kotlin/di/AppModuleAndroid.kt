package di

import android.annotation.SuppressLint
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.PreferenceDataStoreFactory
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.preferencesDataStoreFile
import dispatcher.FolderPickerDispatcherImpl
import dispatchers.FolderPickerDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import music.player.NativeContext
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import platform.AudioPlayerImpl
import platform.BackgroundPlaybackImpl
import platform.PanelSettingsImpl
import platform.HotkeySettingsImpl
import platform.SoundSettingsImpl
import platform.SongsLayoutImpl
import platform.interfaces.AudioPlayer
import platform.interfaces.BackgroundPlayback
import platform.interfaces.PanelSettings
import platform.interfaces.HotkeySettings
import platform.interfaces.SoundSettings
import platform.interfaces.SongsLayout
import repository.OfflineRepositoryImplRust
import repository.SongsRepositoryImplRust
import repository.interfaces.OfflineRepository
import repository.interfaces.SongsRepository
import sealedInterfaces.PlaybackFailure

@SuppressLint("UnsafeOptInUsageError")
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
    // Плеер открывает устройство вывода сразу, а cpal на Android не найдёт его, пока
    // Rust не получит Context: поэтому мост ставится здесь, до создания плеера.
    single<AudioPlayer> {
        if (!NativeContext.install(androidContext())) {
            throw PlaybackFailure.Device("Rust не получил Context: устройство вывода не найти")
        }
        AudioPlayerImpl()
    }
    single<BackgroundPlayback> { BackgroundPlaybackImpl(androidContext()) }
    singleOf(::SongsLayoutImpl) bind SongsLayout::class
    singleOf(::SoundSettingsImpl) bind SoundSettings::class
    singleOf(::HotkeySettingsImpl) bind HotkeySettings::class
    singleOf(::PanelSettingsImpl) bind PanelSettings::class
    singleOf(::SongsRepositoryImplRust) bind SongsRepository::class
    singleOf(::OfflineRepositoryImplRust) bind OfflineRepository::class
    singleOf(::FolderPickerDispatcherImpl) bind FolderPickerDispatcher::class
}
