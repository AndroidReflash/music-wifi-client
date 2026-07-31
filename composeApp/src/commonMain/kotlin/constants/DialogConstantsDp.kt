package constants

import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/** Величины окон поверх экрана, у которых тип `Dp`. */
sealed class DialogConstantsDp(val value: Dp) {

    /**
     * Ширина окна настроек и окна выбора папки. Она у них общая: оба открываются из одного места
     * и должны быть одного размера.
     *
     * Число подобрано под окно выбора папки, которое было первым: на телефоне это почти вся
     * ширина, на десктопе — окно, которое не закрывает собой всё.
     */
    data object Width : DialogConstantsDp(340.dp)

    /** Высота тех же окон. */
    data object Height : DialogConstantsDp(460.dp)

    /**
     * Сколько отведено самому списку внутри окна: остальное занимают заголовок и кнопки.
     * Задавать надо именно список, а не окно: у окна высота теперь своя.
     */
    data object ListHeight : DialogConstantsDp(300.dp)
}
