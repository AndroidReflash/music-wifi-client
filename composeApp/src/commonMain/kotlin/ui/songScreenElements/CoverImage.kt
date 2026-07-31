package ui.songScreenElements

import androidx.compose.foundation.Image
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.layout.ContentScale

/**
 * Картинка обложки из метаданных дорожки — самый нижний слой: поверх неё рисуются и пластинка, и
 * спектр, и всё остальное.
 *
 * Отведённое место у неё разной формы: на телефоне это почти весь экран, на десктопе — квадрат
 * пластинки. Поэтому картинка обрезается по месту, а не вписывается в него: вписанная оставляла бы
 * поля, и на телефоне это читалось бы как ошибка, а не как решение.
 *
 * Картинки нет — не рисуется ничего, и тогда видно тёмный фон. Отдельная заглушка ей не нужна: фон
 * и есть заглушка.
 */
@Composable
fun CoverImage(
    image: ImageBitmap?,
    modifier: Modifier = Modifier,
) {
    if (image == null) {
        return
    }

    Image(
        bitmap = image,
        contentDescription = "Обложка",
        modifier = modifier,
        contentScale = ContentScale.Crop,
    )
}
