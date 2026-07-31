package platform.interfaces

import enums.HotkeyAction

/**
 * Что делает горячая клавиша. Один на все источники нажатий — окно и глобальный перехват зовут
 * одно и то же, и соответствие «действие → вызов плеера» записано в одном месте.
 *
 * Действия идут прямо в репозиторий, мимо ViewModel, — как кнопки гарнитуры и уведомления на
 * Android (`RustMediaPlayer`): это не экран, а ещё один пульт.
 */
interface HotkeyActions {

    fun perform(action: HotkeyAction)
}
