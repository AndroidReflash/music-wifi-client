package platform.interfaces

import kotlinx.coroutines.flow.StateFlow
import sealedInterfaces.GlobalHotkeysState

/**
 * Горячие клавиши вне окна. Сам следит за настройками — включено ли и что назначено — и
 * перехватывает назначенное, пока включено.
 *
 * Интерфейс лежит в десктопном наборе, а не в общем: у телефона перехватывать нечего, и общий код
 * о нём не знает.
 */
interface GlobalHotkeys {

    val state: StateFlow<GlobalHotkeysState>
}
