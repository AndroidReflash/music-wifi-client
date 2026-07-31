package ui

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

@Composable
fun DeviceRow(
    label: String,
    selected: Boolean,
    color: Color,
    onPick: () -> Unit,
) {
    Text(
        text = label,
        color = if (selected) color else Color.White,
        textAlign = TextAlign.Center,
        modifier = Modifier.fillMaxWidth()
            .clickable { onPick() }
            .padding(vertical = 4.dp),
    )
}