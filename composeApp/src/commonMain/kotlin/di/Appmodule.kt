package di

import org.koin.core.module.dsl.singleOf
import org.koin.core.module.dsl.viewModelOf
import org.koin.dsl.bind
import org.koin.dsl.module
import platform.HotkeyActionsImpl
import platform.interfaces.HotkeyActions
import repository.SettingsRepositoryImpl
import repository.NetworkRepositoryImplRust
import repository.interfaces.SettingsRepository
import repository.interfaces.NetworkRepository
import viewModel.SongsScreenViewModel

val appModule = module {
    viewModelOf(::SongsScreenViewModel)
    singleOf(::NetworkRepositoryImplRust) bind NetworkRepository::class
    singleOf(::SettingsRepositoryImpl) bind SettingsRepository::class
    singleOf(::HotkeyActionsImpl) bind HotkeyActions::class
}