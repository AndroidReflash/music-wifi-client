//! Дорожка с сервера как источник для декодера — и отмена, которой её обрывают.

use std::io::{self, Read, Seek, SeekFrom};
use std::sync::{Arc, Mutex, MutexGuard};

use anyhow::Result;
use symphonia::core::io::MediaSource;
use tokio::task::AbortHandle;

use super::cache::{fetch, StreamCache};
use crate::network::QuicSession;

/// Отмена дорожки. Заводится до открытия и тем, кто командует, а не тем, кто ждёт:
/// поэтому ею обрывается и ожидание ответа сервера, и ожидание байтов — оба ждут на
/// одном накопителе.
///
/// Без неё молчащий сервер держит ждущего до таймаута подключения: у quinn это
/// `max_idle_timeout`, сейчас полторы минуты.
#[derive(Clone)]
pub struct TrackCancel {
    cache: Arc<StreamCache>,
    /// Задача докачки. Появляется, когда дорожку открывают.
    task: Arc<Mutex<Option<AbortHandle>>>,
}

impl TrackCancel {
    pub fn new() -> Self {
        TrackCancel {
            cache: Arc::new(StreamCache::new()),
            task: Arc::new(Mutex::new(None)),
        }
    }

    /// Обрывает дорожку: ждущий получает ошибку сразу, а задача докачки снимается.
    /// Снятая задача роняет приёмную сторону потока, и quinn говорит серверу, что
    /// больше не ждёт от него ничего.
    pub fn cancel(&self) {
        self.cache.fail("дорожка закрыта".to_string());
        if let Some(task) = self.task().take() {
            task.abort();
        }
    }

    fn task(&self) -> MutexGuard<'_, Option<AbortHandle>> {
        self.task
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

/// Дорожка, которую отдаёт сервер по QUIC, в виде источника для декодера.
///
/// В протоколе нет запроса диапазона: сервер умеет отдать файл только целиком и только с
/// начала. Поэтому байты копятся здесь по мере прихода — назад перематываться можно
/// свободно, а вперёд не дальше того, что успело прийти; за этим краем чтение ждёт.
pub struct QuicSource {
    cache: Arc<StreamCache>,
    size: u64,
    pos: u64,
    cancel: TrackCancel,
}

impl QuicSource {
    /// Спрашивает у сервера файл и запускает докачку в фоне. Возвращается, как только
    /// сервер назвал длину, — ждать весь файл не надо.
    ///
    /// Само это ожидание обрывается `cancel.cancel()` из любого другого потока.
    pub fn open(session: &Arc<QuicSession>, path: &str, cancel: &TrackCancel) -> Result<Self> {
        let runtime = session.get_runtime_handle();
        let task = runtime.spawn(fetch(
            session.clone(),
            path.to_string(),
            cancel.cache.clone(),
        ));
        *cancel.task() = Some(task.abort_handle());

        let size = cancel.cache.wait_size()?;
        Ok(QuicSource {
            cache: cancel.cache.clone(),
            size,
            pos: 0,
            cancel: cancel.clone(),
        })
    }
}

impl Read for QuicSource {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read = self.cache.read_at(self.pos, buf)?;
        self.pos += read as u64;
        Ok(read)
    }
}

impl Seek for QuicSource {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let target = match from {
            SeekFrom::Start(offset) => i128::from(offset),
            SeekFrom::End(offset) => i128::from(self.size) + i128::from(offset),
            SeekFrom::Current(offset) => i128::from(self.pos) + i128::from(offset),
        };

        // Дальше конца вставать можно — это разрешено и означает, что следующее чтение
        // вернёт ноль. А вот до начала файла места нет.
        let target = u64::try_from(target).map_err(|_| {
            io::Error::new(io::ErrorKind::InvalidInput, "перемотка до начала файла")
        })?;
        self.pos = target;
        Ok(self.pos)
    }
}

impl MediaSource for QuicSource {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.size)
    }
}

impl Drop for QuicSource {
    fn drop(&mut self) {
        // Задача докачки живёт в рантайме сессии сама по себе; без этого она дотянула бы
        // брошенный файл до конца впустую.
        self.cancel.cancel();
    }
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use super::TrackCancel;

    /// Ради этого всё и затевалось: ждущий байтов отпускается по отмене, а не по тому,
    /// что сервер наконец ответил.
    #[test]
    fn waiting_read_ends_when_the_track_is_closed() {
        let cancel = TrackCancel::new();
        let closer = cancel.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            closer.cancel();
        });

        let mut buf = [0u8; 4];
        let err = cancel
            .cache
            .read_at(0, &mut buf)
            .expect_err("закрытая дорожка должна оборвать чтение");
        assert!(err.to_string().contains("дорожка закрыта"));
    }

    /// То же для ожидания ответа сервера: молчащий сервер не должен держать открытие.
    #[test]
    fn waiting_for_the_length_ends_when_the_track_is_closed() {
        let cancel = TrackCancel::new();
        let closer = cancel.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            closer.cancel();
        });

        let err = cancel
            .cache
            .wait_size()
            .expect_err("закрытая дорожка должна оборвать ожидание длины");
        assert!(err.to_string().contains("дорожка закрыта"));
    }
}
