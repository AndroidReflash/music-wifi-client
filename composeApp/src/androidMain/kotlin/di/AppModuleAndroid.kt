package di

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import music.SongsRepositoryImpl
import org.koin.core.module.dsl.singleOf
import org.koin.dsl.bind
import org.koin.dsl.module
import repository.interfaces.SongsRepository


val appModuleAndroid = module {
    includes(appModule)
    single { CoroutineScope(SupervisorJob() + Dispatchers.Default) }
    singleOf(::SongsRepositoryImpl) bind SongsRepository::class
}
