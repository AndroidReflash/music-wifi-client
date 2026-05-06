package ui.songScreenElements

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties

@Composable
fun ChangeIpDialog(
    color: Color,
    isLocal: Boolean,
    onSwitch: () -> Unit,
    onDismiss :() -> Unit,
    onConfirmOnline: (String) -> Unit,
    onChooseOfflineFolder: () -> Unit,
    currentIp: String,
    folderAddress: String
){
    Dialog(
        onDismissRequest = { onDismiss() },
        properties = DialogProperties(usePlatformDefaultWidth = false)
    ){
        Box(
            Modifier
            .fillMaxHeight(0.4f)
            .fillMaxWidth(0.7f)
            .background(Color.Black)
            .border(1.dp, Color.White),
            contentAlignment = Alignment.Center
        ){
            var text by remember { mutableStateOf("") }
            Column (
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.spacedBy(5.dp)
            ) {
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp, Alignment.CenterHorizontally),
                    modifier = Modifier.fillMaxWidth() // Опционально, если нужно на всю ширину
                ) {
                    Text(
                        text = if (!isLocal) "Онлайн" else "Офлайн",
                        color = Color.White,
                        // Убрали padding bottom, чтобы текст стоял ровно по центру линии со свитчем
                        style = MaterialTheme.typography.bodyLarge
                    )

                    // Расстояние между текстом и переключателем
                    Spacer(modifier = Modifier.width(12.dp))

                    Switch(
                        checked = isLocal,
                        onCheckedChange = { onSwitch() },
                        colors = SwitchDefaults.colors(
                            // Цвет "кружка" (thumb)
                            checkedThumbColor = Color.White,
                            uncheckedThumbColor = Color.White,

                            // Цвет дорожки (track)
                            checkedTrackColor = color,
                            uncheckedTrackColor = Color.Gray, // Или другой цвет для выключенного состояния

                            // Цвет обводки (для выключенного состояния, если нужно)
                            uncheckedBorderColor = Color.Gray
                        )
                    )
                }
                if(!isLocal){
                    Text("Текущий айпи: $currentIp", color = Color.White,
                        modifier =  Modifier.padding(bottom = 10.dp)
                    )
                }else{
                    Text(" ", color = Color.White,
                        modifier =  Modifier.padding(bottom = 10.dp)
                    )
                }
                if(!isLocal){
                    Text("Введите айпи сервера", color = Color.White)
                    Text("(пример: http://192.168.100.9:9090)", color = Color.White)
                    BasicTextField(
                        value = text,
                        onValueChange = { text = it },
                        textStyle = TextStyle(color = Color.White),
                        modifier = Modifier
                            .background(Color.Black).fillMaxWidth(0.9f)
                            .border(1.dp, Color.White)
                            .padding(8.dp)
                    )
                    Box(Modifier.border(1.dp, color).clickable{
                        onConfirmOnline(text)
                    }){
                        Text("Применить", color = Color.White, modifier = Modifier.padding(5.dp))
                    }
                }else{
                    Text("Текущая папка: \n $folderAddress", color = Color.White,
                        modifier =  Modifier.padding(bottom = 10.dp)
                    )
                    Box(Modifier.border(1.dp, color).clickable{
                        onChooseOfflineFolder()
                    }){
                        Text("Выбрать папку с треками",
                            color = Color.White,
                            modifier = Modifier.padding(5.dp)
                        )
                    }
                }
            }
        }
    }
}