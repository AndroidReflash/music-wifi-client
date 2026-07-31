//! Чем подключение к серверу может не ответить.
//!
//! Вид беды известен по тому, откуда она пришла, а не по её тексту. Текст внутри нужен
//! человеку и журналу, а решение Kotlin принимает по виду.

use std::error::Error;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, uniffi::Error)]
#[uniffi(with_try_read)]
pub enum NetworkError {
    /// Адрес не разобран: это ошибка того, кто звал, и другой сервер тут ни при чём.
    Address { reason: String },
    /// Подключиться не вышло: сокет не открылся, сервер не ответил, рукопожатие не
    /// состоялось. Другой адрес или повтор позже могут получиться.
    Connect { reason: String },
}

impl Display for NetworkError {
    fn fmt(&self, out: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::Address { reason } => write!(out, "{reason}"),
            NetworkError::Connect { reason } => write!(out, "сервер: {reason}"),
        }
    }
}

impl Error for NetworkError {}
