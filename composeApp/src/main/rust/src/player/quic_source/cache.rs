//! Накопитель байтов дорожки и докачка, которая его наполняет.

use std::io;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};

use crate::network::QuicSession;

/// Сколько байт забирать из потока за раз.
const CHUNK: usize = 64 * 1024;

/// Спрашивает файл и тянет его в накопитель, пока он не кончится.
///
/// Собственного признака «пора уходить» у неё нет: её снимают снаружи, и снятая задача
/// перестаёт существовать на ближайшем ожидании. Тому, кто ждёт накопитель, об этом уже
/// сказано отменой — иначе он ждал бы байтов от задачи, которой нет.
pub(super) async fn fetch(session: Arc<QuicSession>, path: String, cache: Arc<StreamCache>) {
    let (size, mut recv) = match session.open_track(&path).await {
        Ok(opened) => opened,
        Err(err) => {
            cache.fail(format!("{err:#}"));
            return;
        }
    };

    // Файл лежит в памяти целиком, поэтому его длина обязана быть адресуемой:
    // на armeabi-v7a это 32 бита.
    let capacity = match usize::try_from(size) {
        Ok(capacity) => capacity,
        Err(_) => {
            cache.fail(format!("файл в {size} байт в память целиком не берётся"));
            return;
        }
    };
    cache.set_size(size, capacity);

    let mut buf = vec![0u8; CHUNK];
    let mut remaining = size;
    while remaining > 0 {
        match recv.read(&mut buf).await {
            Ok(Some(read)) if read > 0 => {
                let take = read.min(remaining as usize);
                cache.push(&buf[..take]);
                remaining -= take as u64;
            }
            Ok(_) => {
                cache.fail(format!("сервер оборвал файл, не дослав {remaining} байт"));
                return;
            }
            Err(err) => {
                cache.fail(format!("чтение из потока: {err}"));
                return;
            }
        }
    }

    cache.finish();
}

/// Накопитель байтов дорожки: пишет в него докачка, читают декодер и открытие. Всё
/// ожидание дорожки собрано здесь, поэтому и обрывается оно одним движением.
pub(super) struct StreamCache {
    state: Mutex<CacheState>,
    arrived: Condvar,
}

struct CacheState {
    data: Vec<u8>,
    /// Длина файла, как её назвал сервер. До его ответа неизвестна.
    size: Option<u64>,
    /// Файл дошёл до конца.
    done: bool,
    /// Докачка оборвалась или дорожку закрыли, и дальше ждать нечего.
    error: Option<String>,
}

impl StreamCache {
    pub(super) fn new() -> Self {
        StreamCache {
            state: Mutex::new(CacheState {
                data: Vec::new(),
                size: None,
                done: false,
                error: None,
            }),
            arrived: Condvar::new(),
        }
    }

    /// Сервер назвал длину. Место под файл занимается сразу: иначе докачка
    /// перекладывала бы растущий буфер на каждом удвоении.
    fn set_size(&self, size: u64, capacity: usize) {
        let mut state = self.lock();
        state.data.reserve_exact(capacity);
        state.size = Some(size);
        self.arrived.notify_all();
    }

    /// Ждёт, пока сервер назовёт длину файла.
    pub(super) fn wait_size(&self) -> io::Result<u64> {
        let mut state = self.lock();
        loop {
            if let Some(error) = &state.error {
                return Err(io::Error::other(error.clone()));
            }
            if let Some(size) = state.size {
                return Ok(size);
            }

            state = self
                .arrived
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn push(&self, chunk: &[u8]) {
        self.lock().data.extend_from_slice(chunk);
        self.arrived.notify_all();
    }

    fn finish(&self) {
        self.lock().done = true;
        self.arrived.notify_all();
    }

    pub(super) fn fail(&self, message: String) {
        let mut state = self.lock();
        state.error = Some(message);
        state.done = true;
        self.arrived.notify_all();
    }

    /// Копирует байты начиная с `pos`, дождавшись их прихода. Ноль означает конец файла.
    pub(super) fn read_at(&self, pos: u64, buf: &mut [u8]) -> io::Result<usize> {
        let mut state = self.lock();
        loop {
            if let Some(error) = &state.error {
                return Err(io::Error::other(error.clone()));
            }

            if pos < state.data.len() as u64 {
                let start = pos as usize;
                let end = (start + buf.len()).min(state.data.len());
                buf[..end - start].copy_from_slice(&state.data[start..end]);
                return Ok(end - start);
            }

            if state.done {
                return Ok(0);
            }

            state = self
                .arrived
                .wait(state)
                // Замок отравлен, если докачка упала с паникой; накопленное при этом цело.
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn lock(&self) -> MutexGuard<'_, CacheState> {
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use super::StreamCache;

    #[test]
    fn read_waits_for_bytes_to_arrive() {
        let cache = Arc::new(StreamCache::new());
        let writer = cache.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            writer.push(&[1, 2, 3, 4]);
            writer.finish();
        });

        let mut buf = [0u8; 4];
        assert_eq!(cache.read_at(0, &mut buf).expect("чтение"), 4);
        assert_eq!(buf, [1, 2, 3, 4]);

        // За концом файла чтение возвращает ноль, а не ждёт вечно.
        assert_eq!(cache.read_at(4, &mut buf).expect("чтение"), 0);
    }

    #[test]
    fn reads_from_the_middle_of_what_arrived() {
        let cache = StreamCache::new();
        cache.push(&[0, 1, 2, 3, 4, 5]);
        cache.finish();

        let mut buf = [0u8; 3];
        assert_eq!(cache.read_at(2, &mut buf).expect("чтение"), 3);
        assert_eq!(buf, [2, 3, 4]);

        // Просят больше, чем есть с этого места, — отдаётся сколько есть.
        let mut tail = [0u8; 4];
        assert_eq!(cache.read_at(4, &mut tail).expect("чтение"), 2);
        assert_eq!(&tail[..2], &[4, 5]);
    }

    #[test]
    fn broken_download_shows_up_as_an_error() {
        let cache = Arc::new(StreamCache::new());
        let writer = cache.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            writer.fail("связь оборвалась".to_string());
        });

        let mut buf = [0u8; 4];
        let err = cache
            .read_at(0, &mut buf)
            .expect_err("ошибка докачки должна дойти до читателя");
        assert!(err.to_string().contains("связь оборвалась"));
    }
}
