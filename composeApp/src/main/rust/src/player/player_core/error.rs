//! Чем плеер может не ответить.
//!
//! Вид беды известен по тому, откуда ошибка пришла, а не по её тексту: разбирать текст —
//! не опора. Текст внутри нужен человеку и журналу, а решение Kotlin принимает по виду.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

/// `with_try_read` просит uniffi научиться читать ошибку обратно из буфера. Для
/// возвращаемого значения это не нужно, но ошибка лежит полем в [`super::api::PlayerEvent`],
/// а всякое поле перечисления должно уметь и туда, и обратно.
#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
#[uniffi(with_try_read)]
pub enum PlayerError {
    /// Устройство вывода не открылось или отказало: играть не на чем, и следующая
    /// дорожка этого не исправит.
    Device { reason: String },
    /// Дорожка не открылась или оборвалась: нет файла, неизвестный контейнер, молчит
    /// сервер. Другие дорожки при этом могут играть.
    Track { reason: String },
    /// Довод вызова не годится: номера дорожки нет в списке, очередь пуста. Это ошибка
    /// того, кто звал, а не плеера.
    BadArgument { reason: String },
    /// Плеер не поднялся или больше не отвечает: его поток кончился. Лечится только
    /// новым плеером.
    Gone { reason: String },
}

impl PlayerError {
    /// Беда с дорожкой, как её видит декодер: цепочка причин сворачивается в строку
    /// здесь, потому что дальше её никто не разберёт.
    pub fn track(err: anyhow::Error) -> Self {
        PlayerError::Track {
            reason: format!("{err:#}"),
        }
    }

    pub fn device(err: anyhow::Error) -> Self {
        PlayerError::Device {
            reason: format!("{err:#}"),
        }
    }

    pub fn gone(what: &str) -> Self {
        PlayerError::Gone {
            reason: what.to_string(),
        }
    }

    pub fn bad_argument(err: anyhow::Error) -> Self {
        PlayerError::BadArgument {
            reason: format!("{err:#}"),
        }
    }
}

impl Display for PlayerError {
    fn fmt(&self, out: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PlayerError::Device { reason } => write!(out, "устройство вывода: {reason}"),
            PlayerError::Track { reason } => write!(out, "дорожка: {reason}"),
            PlayerError::BadArgument { reason } => write!(out, "{reason}"),
            PlayerError::Gone { reason } => write!(out, "{reason}"),
        }
    }
}

impl Error for PlayerError {}
