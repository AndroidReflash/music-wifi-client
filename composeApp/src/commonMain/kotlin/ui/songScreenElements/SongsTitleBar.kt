package ui.songScreenElements

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * Название играющей дорожки бегущей строкой. Нажатие на неё просят прокрутить список к этой
 * дорожке — саму прокрутку делает тот, кто владеет состоянием списка.
 */
@Composable
fun SongsTitleBar(
    title: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Box(
        modifier.fillMaxWidth()
            .height(30.dp)
            .clickable { onClick() },
        contentAlignment = Alignment.Center
    ) {
        MarqueeTrackTextBounce(title, Modifier.fillMaxWidth(), 15.sp)
    }
}
