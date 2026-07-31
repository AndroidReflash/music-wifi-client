package ui.songScreenElements

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import dataClasses.TopBarDimensions
import musicwifi.composeapp.generated.resources.Res
import musicwifi.composeapp.generated.resources.close
import musicwifi.composeapp.generated.resources.menu
import musicwifi.composeapp.generated.resources.search
import org.jetbrains.compose.resources.painterResource

/**
 * Полоса сверху: либо строка поиска, либо строка состояния — они занимают одно место и
 * показываются по очереди.
 *
 * Состояние поиска сюда приходит снаружи, а не живёт здесь: по запросу отбирается список
 * треков, то есть его знает не только эта полоса.
 *
 * [currentNumber] — номер играющей дорожки, как его видит человек (с единицы), или `null`,
 * если не играет ничего. [trackCount] — сколько дорожек в том списке, который сейчас
 * показывают.
 *
 * Рядом с меню — кнопка эквалайзера, [onEqualizer].
 */
@Composable
fun SongsTopBar(
    textColor: Color,
    interfaceColor: Color,
    isLocal: Boolean,
    isServerAvailable: Boolean,
    trackCount: Int,
    currentNumber: Int?,
    isSearchActive: Boolean,
    searchQuery: String,
    onSearchActiveChange: (Boolean) -> Unit,
    onSearchQueryChange: (String) -> Unit,
    onMenu: () -> Unit,
    onEqualizer: () -> Unit,
    onTextEntry: (Boolean) -> Unit,
    dimensions: TopBarDimensions = TopBarDimensions(),
) {
    val searchFocusRequester = remember { FocusRequester() }

    LaunchedEffect(isSearchActive) {
        if (isSearchActive) {
            searchFocusRequester.requestFocus()
        }
    }

    if (isSearchActive) {
        // Поле уходит из разметки вместе с поиском, и сказать «больше не в фокусе» оно может не
        // успеть: без этого набор текста считался бы идущим и после закрытия поиска.
        DisposableEffect(Unit) {
            onDispose { onTextEntry(false) }
        }
        Row(
            Modifier.fillMaxWidth().height(dimensions.height)
                .padding(start = dimensions.horizontalPadding, end = dimensions.horizontalPadding),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(dimensions.searchSpacing)
        ) {
            OutlinedIcon(
                painter = painterResource(Res.drawable.search),
                contentDescription = "Поиск",
                tint = textColor,
                modifier = Modifier.size(dimensions.searchIcon),
            )
            Box(
                Modifier.weight(1f).fillMaxHeight(),
                contentAlignment = Alignment.CenterStart
            ) {
                if (searchQuery.isEmpty()) {
                    Text(
                        "Поиск треков...",
                        color = textColor.copy(alpha = 0.3f),
                        fontSize = dimensions.searchFontSize
                    )
                }
                BasicTextField(
                    value = searchQuery,
                    onValueChange = onSearchQueryChange,
                    textStyle = TextStyle(color = textColor, fontSize = dimensions.searchFontSize),
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth().padding(end = 28.dp)
                        .focusRequester(searchFocusRequester)
                        .onFocusChanged { state -> onTextEntry(state.isFocused) }
                )
            }
            OutlinedIcon(
                painter = painterResource(Res.drawable.close),
                contentDescription = "Закрыть поиск",
                tint = textColor,
                modifier = Modifier.size(dimensions.closeIcon).clickable {
                    onSearchQueryChange("")
                    onSearchActiveChange(false)
                },
            )
        }
    } else {
        Row(
            Modifier.fillMaxWidth().height(dimensions.height)
                .padding(start = dimensions.horizontalPadding, end = dimensions.horizontalPadding),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Row(
                Modifier.weight(1f),
                horizontalArrangement = Arrangement.spacedBy(dimensions.statusSpacing),
                verticalAlignment = Alignment.CenterVertically
            ) {
                if (!isLocal) {
                    Box(
                        Modifier.size(dimensions.statusDot)
                            .clip(RoundedCornerShape(dimensions.statusDot / 2))
                            .background(if (isServerAvailable) interfaceColor else textColor)
                    )
                    Text(
                        if (isServerAvailable) "В сети" else "Недоступен",
                        color = textColor,
                        fontSize = dimensions.fontSize
                    )
                    if (isServerAvailable) {
                        SearchIcon(textColor, dimensions.searchIcon, onSearchActiveChange)
                    }
                } else {
                    SearchIcon(textColor, dimensions.searchIcon, onSearchActiveChange)
                }
            }
            Row(
                Modifier.weight(1f),
                horizontalArrangement = Arrangement.spacedBy(
                    dimensions.statusSpacing,
                    Alignment.CenterHorizontally,
                ),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                OutlinedIcon(
                    painter = painterResource(Res.drawable.menu),
                    contentDescription = "Всплывашка",
                    tint = textColor,
                    modifier = Modifier.size(dimensions.menuIcon).clickable { onMenu() },
                )
                Canvas(Modifier.size(dimensions.menuIcon).clickable { onEqualizer() }) {
                    // Обводка, как у соседних значков: под панелью может быть картинка обложки.
                    equalizerGlyph(Color.Black, extra = 2.dp.toPx())
                    equalizerGlyph(textColor, extra = 0f)
                }
            }
            OutlinedText(
                text = if (currentNumber != null) "$currentNumber / $trackCount" else "$trackCount",
                color = textColor,
                modifier = Modifier.weight(1f),
                textModifier = Modifier.fillMaxWidth(),
                fontSize = dimensions.fontSize,
                textAlign = TextAlign.End,
            )
        }
    }
}

/**
 * Значок эквалайзера: три вертикальные черты с бегунками на разной высоте. [extra] утолщает линии —
 * так рисуется чёрная подложка-обводка под цветным значком.
 */
private fun DrawScope.equalizerGlyph(color: Color, extra: Float) {
    val thin = size.minDimension * 0.08f + extra
    val knob = size.minDimension * 0.11f + extra / 2
    val places = listOf(0.25f, 0.5f, 0.75f)
    val knobs = listOf(0.35f, 0.68f, 0.45f)

    places.forEachIndexed { index, place ->
        val x = size.width * place
        drawLine(
            color = color,
            start = Offset(x, size.height * 0.15f),
            end = Offset(x, size.height * 0.85f),
            strokeWidth = thin,
            cap = StrokeCap.Round,
        )
        drawCircle(color = color, radius = knob, center = Offset(x, size.height * knobs[index]))
    }
}

@Composable
private fun SearchIcon(textColor: Color, size: Dp, onActive: (Boolean) -> Unit) {
    OutlinedIcon(
        painter = painterResource(Res.drawable.search),
        contentDescription = "Поиск",
        tint = textColor,
        modifier = Modifier.size(size).clickable { onActive(true) },
    )
}
