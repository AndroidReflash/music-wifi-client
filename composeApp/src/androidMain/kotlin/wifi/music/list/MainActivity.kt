package wifi.music.list


import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.annotation.RequiresApi
import androidx.compose.ui.graphics.Color
import dispatcher.FolderPickerEffectHandler
import dispatcher.checkStoragePermission
import dispatcher.requestStoragePermission
import ui.SongsScreen

class MainActivity : ComponentActivity() {
    @RequiresApi(Build.VERSION_CODES.TIRAMISU)
    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        
        val check = checkStoragePermission(this@MainActivity)
        if(!check){
            requestStoragePermission(this@MainActivity)
        }
        setContent {
            val textColor = Color.White
            val backgroundColor = Color.Black
            SongsScreen(
                textColor = textColor,
                backgroundColor = backgroundColor
            )
            FolderPickerEffectHandler(
                textColor = textColor,
                backgroundColor = backgroundColor
            )
        }
    }
}
