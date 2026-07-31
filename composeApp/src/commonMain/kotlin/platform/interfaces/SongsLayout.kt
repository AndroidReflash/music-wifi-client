package platform.interfaces

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import dataClasses.TopBarDimensions
import enums.VisualizerShape

/**
 * Расстановка экрана воспроизведения. На телефоне всё в один столбец, на десктопе — в две
 * половины; сами блоки при этом одни и те же.
 *
 * Живёт здесь, а не в `ui/`, по правилу шаблона: интерфейс с `@Composable`-методом — это и есть
 * замена `expect @Composable fun`, которой в Compose нет.
 *
 * Блоки приходят готовыми, отдельными кусками разметки, и расстановка их только размещает:
 * каждому она передаёт свой `Modifier`, потому что вес в столбце и размер в строке знает она, а
 * не блок. Состояние блоки получают там, где их собрали, — здесь его нет вовсе.
 *
 * [listState] нужен для полосы прокрутки: она есть только на десктопе и только там её строят.
 *
 * [coverImage] — картинка обложки, самый нижний слой. Места под неё отведены разные: на телефоне
 * это всё, что выше панели управления, на десктопе — квадрат пластинки. Решает это расстановка, она
 * одна и знает, где у неё что.
 *
 * [panelDivider] и [panelTransparent] — телефонные: на десктопе панель управления ни на чём не
 * лежит, а картинка живёт в квадрате пластинки, и обе настройки там ничего не значат.
 *
 * [visualizer], [cover] и [coverImage] — `null`, когда их велено не показывать. Именно `null`, а не пустой кусок
 * разметки: место под спрятанное занимать не надо, и убрать его может только расстановка.
 *
 * [shape] выбирает человек в настройках, а расстановка по ней решает, сколько места спектру дать:
 * кольцу нужен квадрат, полосам — ширина.
 */
interface SongsLayout {

    @Composable
    fun Content(
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
    )
}
