package sealedInterfaces

/**
 * Чем плеер может не ответить. Вид беды известен по тому, откуда она пришла, а не по
 * тексту: разбирать текст — не опора. Текст нужен человеку и журналу.
 */
sealed class PlaybackFailure(val reason: String) : Exception(reason) {

    /** Устройство вывода не открылось или отказало: следующая дорожка этого не исправит. */
    class Device(reason: String) : PlaybackFailure(reason)

    /** Дорожка не открылась или оборвалась. Другие дорожки при этом могут играть. */
    class Track(reason: String) : PlaybackFailure(reason)

    /** Довод вызова не годится: номера дорожки нет в списке, очередь пуста. */
    class BadArgument(reason: String) : PlaybackFailure(reason)

    /** Плеера больше нет: его поток кончился. Лечится только новым плеером. */
    class Gone(reason: String) : PlaybackFailure(reason)

    /**
     * До сервера не достучались. Свои файлы при этом играют: это беда подключения, а не
     * плеера, и её приносит не Rust, а попытка подключиться.
     */
    class Server(reason: String) : PlaybackFailure(reason)
}
