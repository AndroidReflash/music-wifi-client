package dispatcher

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import dispatchers.FolderPickerDispatcher
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel
import viewModel.SongsScreenViewModel
import android.os.Environment
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.runtime.collectAsState

@Composable
fun FolderPickerEffectHandler(
    viewModel: SongsScreenViewModel = koinViewModel(),
    dispatcher: FolderPickerDispatcher = koinInject(),
    textColor: Color = Color.White,
    backgroundColor: Color = Color.Black,
) {
    val context = LocalContext.current
    val interfaceColor by viewModel.interfaceColor.collectAsState()
    var showCustomFolderPicker by remember { mutableStateOf(false) }

    LaunchedEffect(dispatcher) {
        dispatcher.events.collect {
            if (checkStoragePermission(context)) {
                showCustomFolderPicker = true
            } else {
                requestStoragePermission(context)
            }
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
            onDismiss = { showCustomFolderPicker = false }
        )
    }
}

@Composable
private fun SimpleFolderPickerDialog(
    interfaceColor: Color,
    textColor: Color,
    backgroundColor: Color,
    onFolderSelected: (String) -> Unit,
    onDismiss: () -> Unit
) {
    var currentDir by remember {
        mutableStateOf(Environment.getExternalStorageDirectory())
    }

    val subFolders = remember(currentDir) {
        currentDir.listFiles { file -> file.isDirectory && !file.name.startsWith(".") }
            ?.sortedBy { it.name.lowercase() } ?: emptyList()
    }

    AlertDialog(
        modifier = Modifier.border(1.dp, interfaceColor),
        containerColor = backgroundColor,
        onDismissRequest = onDismiss,
        title = { Text("Выбери папку с музыкой", color = textColor) },
        text = {
            Column{
                Text("Текущий путь: ${currentDir.absolutePath}", style = MaterialTheme.typography.bodySmall, color = textColor)
                Spacer(modifier = Modifier.height(8.dp))

                LazyColumn(modifier = Modifier.height(300.dp)) {
                    if (currentDir.parentFile != null && currentDir != Environment.getExternalStorageDirectory()) {
                        item {
                            TextButton(onClick = { currentDir = currentDir.parentFile }) {
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
                                .padding(vertical = 12.dp, horizontal = 8.dp)
                            , color = textColor
                        )
                    }
                }
            }
        },
        confirmButton = {
            TextButton(onClick = { onFolderSelected(currentDir.absolutePath) }) { Text("Применить", color = interfaceColor) }
        },
        dismissButton = {
            TextButton(onClick = onDismiss) { Text("Отмена", color = textColor) }
        }
    )
}
