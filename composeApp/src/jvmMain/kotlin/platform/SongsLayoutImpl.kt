package platform

import androidx.compose.foundation.LocalScrollbarStyle
import androidx.compose.foundation.VerticalScrollbar
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.rememberScrollbarAdapter
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import dataClasses.TopBarDimensions
import enums.VisualizerShape
import platform.interfaces.SongsLayout

/**
 * Десктопная расстановка: полоса сверху, под ней окно поделено пополам. Слева список дорожек с
 * полосой прокрутки, справа визуализация и всё управление.
 *
 * Полоса прокрутки есть только здесь: `VerticalScrollbar` живёт в наборе skiko, для Android его
 * нет вовсе, поэтому и вёрстка десктопа не может лежать в общем коде.
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
        // Телефонные настройки: здесь панель ни на чём не лежит, а картинка живёт в квадрате.
        panelDivider: Boolean,
        panelTransparent: Boolean,
    ) {
        Column(Modifier.fillMaxSize().background(backgroundColor)) {
            topBar(DESKTOP_TOP_BAR)

            Row(Modifier.fillMaxWidth().weight(1f)) {
                Row(Modifier.weight(1f).fillMaxHeight()) {
                    // Цвет полосе задаётся свой: по умолчанию она чёрная с прозрачностью, а фон
                    // здесь тоже чёрный — её было бы не видно.
                    VerticalScrollbar(
                        adapter = rememberScrollbarAdapter(listState),
                        modifier = Modifier.fillMaxHeight().padding(horizontal = 4.dp),
                        style = LocalScrollbarStyle.current.copy(
                            unhoverColor = interfaceColor.copy(alpha = 0.4f),
                            hoverColor = interfaceColor,
                        ),
                    )
                    list(Modifier.weight(1f).fillMaxHeight())
                }

                // Отступ слева — зазор между половинами, справа — от края окна.
                // Выравнивание по центру нужно ползунку: он занимает 90% ширины и без него
                // прижимался бы к левому краю половины.
                Column(
                    Modifier.weight(1f).fillMaxHeight()
                        .padding(start = 10.dp, end = 10.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    // Квадрат по меньшей стороне свободного места, по центру: в нём обложка
                    // нижним слоем, визуализатор поверх неё. Сторона считается здесь, а не
                    // задаётся отношением сторон, потому что отношение примеряется к одной оси и
                    // на низком окне квадрат вылез бы за край.
                    //
                    // Спрятали и то и другое — квадрата нет вовсе, и управление занимает всю
                    // половину.
                    if (visualizer != null || cover != null || coverImage != null) {
                        BoxWithConstraints(
                            Modifier.fillMaxWidth().weight(1f),
                            contentAlignment = Alignment.Center
                        ) {
                            Box(Modifier.size(minOf(maxWidth, maxHeight))) {
                                // Картинка обложки — в размер квадрата и ниже всего остального.
                                coverImage?.invoke(Modifier.fillMaxSize())
                                // Пластинка в полную силу: поверх картинки, но под спектром.
                                cover?.invoke(Modifier.fillMaxSize(), interfaceColor)
                                // А визуализатор поверх неё — приглушённым: в полную силу он бьёт
                                // по глазам и скрыл бы обложку под собой. Это то число, которое
                                // подбирают глазами.
                                // Кольцу нужен весь квадрат — тогда оно соосно пластинке под
                                // ним; полосам хватает ширины, высоту они задают себе сами.
                                val spectrum = when (shape) {
                                    VisualizerShape.RING -> Modifier.fillMaxSize()
                                    VisualizerShape.BARS ->
                                        Modifier.fillMaxWidth().align(Alignment.Center)
                                }
                                visualizer?.invoke(
                                    spectrum,
                                    interfaceColor.copy(alpha = 0.3f),
                                )
                            }
                        }
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
 * Размеры полосы сверху для десктопа: вдвое против телефонных.
 *
 * Размер текста на телефоне не задан — `Text` берёт его из стиля, и это 14sp по умолчанию
 * Compose. Здесь поэтому стоит 28sp: если на глаз окажется не вдвое, крутить надо это число.
 */
private val DESKTOP_TOP_BAR = TopBarDimensions(
    height = 60.dp,
    horizontalPadding = 10.dp,
    searchSpacing = 16.dp,
    statusSpacing = 10.dp,
    searchIcon = 36.dp,
    closeIcon = 40.dp,
    menuIcon = 40.dp,
    statusDot = 30.dp,
    searchFontSize = 28.sp,
    fontSize = 28.sp,
)
