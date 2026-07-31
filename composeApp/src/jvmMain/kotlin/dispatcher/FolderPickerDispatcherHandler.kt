package dispatcher

import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogProperties
import constants.DialogConstantsDp
import dispatchers.FolderPickerDispatcher
import java.io.File
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel
import viewModel.SongsScreenViewModel

/**
 * Ответ на просьбу выбрать папку с музыкой. Пакет и имя те же, что в `androidMain`: общий код
 * не должен знать, какая половина ему досталась.
 *
 * От андроидного двойника отличается двумя вещами, и обе — от платформы. Начинают отсюда с
 * домашнего каталога, а не с корня внешней памяти, которого на десктопе нет. И разрешения не
 * спрашиваются: на чтение своих файлов их здесь никто не требует.
 */
@Composable
fun FolderPickerEffectHandler(
    viewModel: SongsScreenViewModel = koinViewModel(),
    dispatcher: FolderPickerDispatcher = koinInject(),
    textColor: Color = Color.White,
    backgroundColor: Color = Color.Black,
) {
    val interfaceColor by viewModel.interfaceColor.collectAsState()
    var showCustomFolderPicker by remember { mutableStateOf(false) }

    LaunchedEffect(dispatcher) {
        dispatcher.events.collect {
            showCustomFolderPicker = true
        }
    }

    if (showCustomFolderPicker) {
        SimpleFolderPickerDialog(
            interfaceColor = interfaceColor,
            textColor = textColor,
            backgroundColor = backgroundColor,
            onFolderSelected = { path ->
                showCustomFolderPicker = false
                viewModel.onFolderSelected(path)
            },
            onDismiss = { showCustomFolderPicker = false },
        )
    }
}

@Composable
private fun SimpleFolderPickerDialog(
    interfaceColor: Color,
    textColor: Color,
    backgroundColor: Color,
    onFolderSelected: (String) -> Unit,
    onDismiss: () -> Unit,
) {
    var currentDir by remember { mutableStateOf(File(System.getProperty("user.home"))) }

    val subFolders = remember(currentDir) {
        currentDir.listFiles { file -> file.isDirectory && !file.name.startsWith(".") }
            ?.sortedBy { it.name.lowercase() } ?: emptyList()
    }

    AlertDialog(
        // Размер общий с окном настроек: оба открываются из одного места, и разные размеры
        // читались бы как разные окна одного приложения. Ширину задаём сами, поэтому платформенную
        // по умолчанию надо отключить — иначе она её и оставит.
        modifier = Modifier.border(1.dp, interfaceColor)
            .size(width = DialogConstantsDp.Width.value, height = DialogConstantsDp.Height.value),
        properties = DialogProperties(usePlatformDefaultWidth = false),
        containerColor = backgroundColor,
        onDismissRequest = onDismiss,
        title = { Text("Выбери папку с музыкой", color = textColor) },
        text = {
            Column {
                Text(
                    "Текущий путь: ${currentDir.absolutePath}",
                    style = MaterialTheme.typography.bodySmall,
                    color = textColor,
                )
                Spacer(modifier = Modifier.height(8.dp))

                LazyColumn(modifier = Modifier.height(DialogConstantsDp.ListHeight.value)) {
                    // Вверх — до самого корня файловой системы: на десктопе музыка лежит и
                    // за пределами домашнего каталога, на отдельном диске например.
                    val parent = currentDir.parentFile
                    if (parent != null) {
                        item {
                            TextButton(onClick = { currentDir = parent }) {
                                Text(".. (Назад)", color = textColor)
                            }
                        }
                    }

                    items(subFolders) { folder ->
                        Text(
                            text = "📁 ${folder.name}",
                            modifier = Modifier
                                .fillMaxWidth()
                                .clickable { currentDir = folder }
                                .padding(vertical = 12.dp, horizontal = 8.dp),
                            color = textColor,
                        )
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = { onFolderSelected(currentDir.absolutePath) }) {
                Text("Применить", color = interfaceColor)
            }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Отмена", color = textColor) }
        },
    )
}
