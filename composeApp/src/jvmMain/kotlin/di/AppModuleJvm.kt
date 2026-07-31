package di

import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.PreferenceDataStoreFactory
import androidx.datastore.preferences.core.Preferences
import dispatcher.FolderPickerDispatcherImpl
import dispatchers.FolderPickerDispatcher
import java.io.File
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import org.koin.core.module.dsl.createdAtStart
import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import platform.AudioPlayerImpl
import platform.BackgroundPlaybackImpl
import platform.GlobalHotkeysImpl
import platform.PanelSettingsImpl
import platform.HotkeySettingsImpl
import platform.SoundSettingsImpl
import platform.SongsLayoutImpl
import platform.WindowHotkeysImpl
import platform.interfaces.AudioPlayer
import platform.interfaces.BackgroundPlayback
import platform.interfaces.GlobalHotkeys
import platform.interfaces.PanelSettings
import platform.interfaces.HotkeySettings
import platform.interfaces.SoundSettings
import platform.interfaces.SongsLayout
import platform.interfaces.WindowHotkeys
import repository.OfflineRepositoryImplRust
import repository.SongsRepositoryImplRust
import repository.interfaces.OfflineRepository
import repository.interfaces.SongsRepository

/**
 * Граф зависимостей десктопа: общий модуль плюс то, что даёт платформа.
 *
 * Моста к контексту Android здесь нет и не нужно: на десктопе cpal находит устройство вывода
 * сам, поэтому плеер создаётся одной строкой.
 */
val appModuleJvm = module {
    includes(appModule)

    single<DataStore<Preferences>> {
        PreferenceDataStoreFactory.create(produceFile = { File(settingsDirectory(), SETTINGS_FILE) })
    }

    single { CoroutineScope(SupervisorJob() + Dispatchers.Default) }
    singleOf(::AudioPlayerImpl) bind AudioPlayer::class
    singleOf(::BackgroundPlaybackImpl) bind BackgroundPlayback::class
    singleOf(::SongsLayoutImpl) bind SongsLayout::class
    singleOf(::SoundSettingsImpl) bind SoundSettings::class
    singleOf(::HotkeySettingsImpl) bind HotkeySettings::class
    singleOf(::WindowHotkeysImpl) bind WindowHotkeys::class
    // При запуске, а не при первом обращении: он сам следит за настройкой, и спрашивать его, пока
    // не открыли настройки, некому.
    singleOf(::GlobalHotkeysImpl) { createdAtStart() } bind GlobalHotkeys::class
    singleOf(::PanelSettingsImpl) bind PanelSettings::class
    singleOf(::SongsRepositoryImplRust) bind SongsRepository::class
    singleOf(::OfflineRepositoryImplRust) bind OfflineRepository::class
    singleOf(::FolderPickerDispatcherImpl) bind FolderPickerDispatcher::class
}

/** Имя с расширением, которое `DataStore` ждёт от файла настроек. */
private const val SETTINGS_FILE = "settings.preferences_pb"

/**
 * Где лежат настройки: в корне папки приложения. Приложение носят папкой, и настройки едут вместе
 * с ним. Рабочий каталог для этого не годится — он тот, откуда запустили, и при запуске из другого
 * места настройки начинались бы с чистого листа.
 *
 * Корень находится по `jpackage.app-path`: запускающий файл собранного приложения сам задаёт это
 * свойство путём к себе, а лежит он в `bin/`. При запуске из Gradle (`./gradlew :composeApp:run`)
 * собранного приложения нет и свойства тоже, тогда настройки в рабочем каталоге задачи —
 * `composeApp/`.
 */
private fun settingsDirectory(): File {
    val dbDir = File("database")
    if (!dbDir.exists()) {
        dbDir.mkdirs()
    }
    return dbDir.absoluteFile
}
