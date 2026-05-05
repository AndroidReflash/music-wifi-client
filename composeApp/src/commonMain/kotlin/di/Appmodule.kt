package di

import com.google.gson.GsonBuilder
import okhttp3.OkHttpClient
import org.koin.core.module.dsl.singleOf
import org.koin.core.module.dsl.viewModelOf
import org.koin.dsl.bind
import org.koin.dsl.module
import repository.NetworkRepositoryImpl
import repository.interfaces.NetworkRepository
import viewModel.SongsScreenViewModel
import java.util.concurrent.TimeUnit

val appModule = module {
    single {
        OkHttpClient.Builder()
            .connectTimeout(10, TimeUnit.SECONDS)
            .readTimeout(10, TimeUnit.SECONDS)
            .build()
    }
    single {
        GsonBuilder().create()
    }

    viewModelOf(::SongsScreenViewModel)
    singleOf(::NetworkRepositoryImpl) bind NetworkRepository::class
}