package platform

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.dp
import dataClasses.TopBarDimensions
import enums.VisualizerShape
import platform.interfaces.SongsLayout

/**
 * Телефонная расстановка: всё в один столбец, а картинка обложки, пластинка и полосы спектра —
 * слоями за ним.
 *
 * Слои снизу вверх: картинка, пластинка, спектр, содержимое. Картинка занимает всё, кроме панели
 * управления внизу; пластинка со спектром стоят квадратом по центру экрана, как стояли.
 *
 * Отступ под системные панели здесь и нигде больше: на десктопе таких панелей нет.
 */
class SongsLayoutImpl : SongsLayout {

    @Composable
    override fun Content(
        backgroundColor: Color,
        interfaceColor: Color,
        listState: LazyListState,
        visualizer: (@Composable (Modifier, Color) -> Unit)?,
        shape: VisualizerShape,
        topBar: @Composable (TopBarDimensions) -> Unit,
        list: @Composable (Modifier) -> Unit,
        coverImage: (@Composable (Modifier) -> Unit)?,
        cover: (@Composable (Modifier, Color) -> Unit)?,
        title: @Composable (Modifier) -> Unit,
        controls: @Composable (Modifier) -> Unit,
        slider: @Composable (Modifier) -> Unit,
        panelDivider: Boolean,
        panelTransparent: Boolean,
    ) {
        // Высота панели управления складывается из трёх блоков и числом нигде не записана, поэтому
        // она замеряется. Иначе нижний край картинки пришлось бы задавать константой, и та разошлась
        // бы с панелью при первой же правке любого из блоков.
        var panelHeight by remember { mutableStateOf(0) }
        val density = LocalDensity.current

        Box(Modifier.fillMaxSize().background(backgroundColor), contentAlignment = Alignment.Center) {
            // Картинка занимает всё, кроме системных панелей Android, — и, если панель управления
            // непрозрачна, кроме неё. Высота панели вычитается сверх отступа под кнопки навигации:
            // сама панель стоит в столбце, который от них отодвинут, поэтому её верхний край выше
            // низа экрана на обе величины.
            val underPanel = if (panelTransparent) {
                0.dp
            } else {
                with(density) { panelHeight.toDp() }
            }
            coverImage?.invoke(
                Modifier.fillMaxSize()
                    .systemBarsPadding()
                    .padding(bottom = underPanel)
            )

            // Спрятали всё — слоя нет вовсе, а не пустое место под ним.
            if (visualizer != null || cover != null) {
                BoxWithConstraints(
                    Modifier.fillMaxSize(),
                    contentAlignment = Alignment.Center
                ) {
                    // Экран телефона закреплён вертикально, поэтому меньшая сторона — ширина, и
                    // квадрат выходит во всю ширину: полосы спектра встают там же, где стояли.
                    Box(Modifier.size(minOf(maxWidth, maxHeight))) {
                        cover?.invoke(
                            Modifier.fillMaxSize(),
                            interfaceColor.copy(alpha = BACKGROUND_ALPHA)
                        )
                        // Кольцу нужен весь квадрат — тогда оно соосно пластинке под ним;
                        // полосам хватает ширины, высоту они задают себе сами.
                        val spectrum = when (shape) {
                            VisualizerShape.RING -> Modifier.fillMaxSize()
                            VisualizerShape.BARS -> Modifier.fillMaxWidth().align(Alignment.Center)
                        }
                        visualizer?.invoke(
                            spectrum,
                            interfaceColor.copy(alpha = BACKGROUND_ALPHA),
                        )
                    }
                }
            }

            Column(
                modifier = Modifier.fillMaxSize().systemBarsPadding(),
                horizontalAlignment = Alignment.CenterHorizontally,
                verticalArrangement = Arrangement.Center
            ) {
                topBar(TopBarDimensions())
                if (panelDivider) {
                    PanelLine(interfaceColor)
                }
                list(Modifier.weight(1f))

                // Три нижних блока собраны в один столбец только затем, чтобы их общую высоту
                // можно было замерить одним разом.
                Column(
                    modifier = Modifier.fillMaxWidth()
                        .onSizeChanged { measured -> panelHeight = measured.height },
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    if (panelDivider) {
                        // Линия входит в панель, а не стоит над ней: тогда её толщина попадает в
                        // замер, и картинка кончается над линией, а не под ней.
                        PanelLine(interfaceColor)
                    }
                    title(Modifier)
                    controls(Modifier)
                    slider(Modifier)
                }
            }
        }
    }
}

/**
 * Насколько прозрачны пластинка и полосы спектра. Поверх них стоит весь экран, поэтому в полную
 * силу они мешали бы читать; это то число, которое подбирают глазами.
 */
private const val BACKGROUND_ALPHA = 0.3f

/**
 * Линия, отделяющая полосу — верхнюю или нижнюю — от списка. Обе включаются одной настройкой и
 * обязаны выглядеть одинаково, поэтому нарисованы в одном месте.
 */
@Composable
private fun PanelLine(color: Color) {
    Box(
        Modifier.fillMaxWidth()
            .height(1.dp)
            .background(color)
    )
}
