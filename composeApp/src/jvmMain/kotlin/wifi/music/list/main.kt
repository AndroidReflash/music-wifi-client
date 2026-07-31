package wifi.music.list

import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalWindowInfo
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application
import java.awt.Dimension
import di.appModuleJvm
import dispatcher.FolderPickerEffectHandler
import org.koin.core.context.GlobalContext.startKoin
import platform.interfaces.WindowHotkeys
import ui.SongsScreen
import uniffi.player_core.initRustLogging

fun main() {
    val koin = startKoin { modules(appModuleJvm) }.koin
    val hotkeys = koin.get<WindowHotkeys>()
    initRustLogging()
    application {
        Window(
            onCloseRequest = ::exitApplication,
            title = "musicwifi",
            // Раньше элемента в фокусе, иначе назначенный пробел жал бы последнюю нажатую
            // кнопку. Окна настроек и захвата клавиши — свои окна, и их нажатия сюда не
            // попадают вовсе.
            onPreviewKeyEvent = hotkeys::onPreviewKeyEvent,
        ) {
            val focused = LocalWindowInfo.current.isWindowFocused
            LaunchedEffect(focused) {
                if (!focused) {
                    hotkeys.releaseAll()
                }
            }
            // Окно поделено пополам, и на узком оно рассыпается: обе половины становятся
            // нечитаемыми раньше, чем перестают влезать. Меньше этого не дать.
            LaunchedEffect(Unit) {
                window.minimumSize = Dimension(900, 600)
            }
            SongsScreen(
                textColor = Color.White,
                backgroundColor = Color.Black,
                onTextEntry = hotkeys::setTextEntry,
            )
            // Экран только просит папку, а показывает диалог этот обработчик — так же, как
            // на Android его показывает тот, что рядом с `MainActivity`.
            FolderPickerEffectHandler(
                textColor = Color.White,
                backgroundColor = Color.Black,
            )
        }
    }
}
