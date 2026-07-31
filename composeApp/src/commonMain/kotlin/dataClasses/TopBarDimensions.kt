package dataClasses

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/**
 * Размеры полосы сверху и того, что в ней стоит.
 *
 * Значения по умолчанию — те, что нарисованы на телефоне: полоса вызывается без этого
 * параметра, и на Android ничего не меняется. Десктоп передаёт свой набор.
 *
 * [fontSize] по умолчанию не задан намеренно. `Text` при `Unspecified` берёт размер из стиля,
 * а поставить сюда число значило бы менять вид на Android — сейчас размер там ниоткуда не
 * задаётся, и угадывать его нельзя.
 */
data class TopBarDimensions(
    val height: Dp = 30.dp,
    val horizontalPadding: Dp = 5.dp,
    val searchSpacing: Dp = 8.dp,
    val statusSpacing: Dp = 5.dp,
    val searchIcon: Dp = 18.dp,
    val closeIcon: Dp = 20.dp,
    val menuIcon: Dp = 20.dp,
    val statusDot: Dp = 15.dp,
    val searchFontSize: TextUnit = 14.sp,
    val fontSize: TextUnit = TextUnit.Unspecified,
)
