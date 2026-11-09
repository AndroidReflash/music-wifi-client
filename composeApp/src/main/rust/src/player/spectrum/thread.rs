//! Поток спектра: преобразование Фурье и отдача полос наружу.
//!
//! Зачем отдельный поток. Полосы считаются из тех же кадров, что уходят устройству, и раньше
//! считались там же, где эти кадры готовятся, — в потоке декодирования. У того потока одна
//! обязанность: успевать кормить кольцо вывода, иначе устройство получит тишину и это слышно.
//! Преобразование Фурье и вызов через границу в Kotlin (а это JNI, десятки микросекунд) к этой
//! обязанности отношения не имеют и только мешали ей.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::error;

use crate::player::audioconverter::compute_bands;
use crate::player::fft_processor::analyze_samples;
use crate::player::output::OutputControl;
use crate::player::player_core::BandsListener;

/// Сколько окон может ждать счёта. Для визуализатора допустимо терять кадры, копить их нельзя:
/// при отставании потребителя лишнее выталкивается, а отправитель звука не ждёт.
const QUEUE_CAP: usize = 32;

/// Окно отсчётов, из которого считать полосы.
struct Window {
    samples: Vec<f32>,
    /// Показание счётчика отданных отсчётов, при котором эти полосы зазвучат.
    at: u64,
    /// Поколение: окно от дорожки, которую уже бросили, считать незачем.
    generation: u64,
}

struct Inbox {
    queue: Mutex<VecDeque<Window>>,
    ready: Condvar,
    closed: AtomicBool,
}

impl Inbox {
    fn new() -> Self {
        Inbox {
            queue: Mutex::new(VecDeque::with_capacity(QUEUE_CAP)),
            ready: Condvar::new(),
            closed: AtomicBool::new(false),
        }
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<Window>> {
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn close(&self) {
        self.closed.store(true, Ordering::Release);
        self.ready.notify_all();
    }

    fn push(&self, window: Window) {
        if self.closed.load(Ordering::Acquire) {
            return;
        }
        let mut queue = self.lock();
        while queue.len() >= QUEUE_CAP {
            queue.pop_front();
        }
        queue.push_back(window);
        drop(queue);
        self.ready.notify_one();
    }

    /// Выбрасывает накопленное: при смене дорожки старые окна не должны висеть в памяти до
    /// чтения потребителем.
    fn clear(&self) {
        self.lock().clear();
        self.ready.notify_all();
    }

    /// Ждёт окно не дольше `wait`. `None` — очередь закрыта, поток пора завершать.
    fn take(&self, wait: Duration) -> Option<Option<Window>> {
        let mut queue = self.lock();
        loop {
            if let Some(window) = queue.pop_front() {
                return Some(Some(window));
            }
            if self.closed.load(Ordering::Acquire) {
                return None;
            }
            let (guard, result) = self
                .ready
                .wait_timeout(queue, wait)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            queue = guard;
            if result.timed_out() {
                return Some(queue.pop_front());
            }
        }
    }
}

/// Ручка потока спектра у того, кто его кормит.
pub struct SpectrumThread {
    inbox: Arc<Inbox>,
    generation: Arc<AtomicU64>,
    worker: Option<JoinHandle<()>>,
}

impl SpectrumThread {
    pub fn start(bands: Box<dyn BandsListener>, control: OutputControl) -> Self {
        let inbox = Arc::new(Inbox::new());
        let generation = Arc::new(AtomicU64::new(0));
        let worker_inbox = inbox.clone();
        let worker_generation = generation.clone();

        // Поток спектра не обязателен для звука, поэтому его неудача не обрывает создание плеера.
        // Но и молчать о ней нельзя: без него визуализатор просто не появится, и это надо будет
        // откуда-то узнать.
        let worker = match thread::Builder::new()
            .name("spectrum".to_string())
            .spawn(move || Spectrum::new(bands, control, worker_generation).serve(worker_inbox))
        {
            Ok(worker) => Some(worker),
            Err(err) => {
                error!("поток спектра не запустился, визуализатора не будет: {err}");
                None
            }
        };

        SpectrumThread {
            inbox,
            generation,
            worker,
        }
    }

    /// Отдаёт окно на счёт. Отправка не ждёт звука: при заполнении старое окно вытесняется,
    /// терять кадры визуализатора допустимо.
    pub fn feed(&self, samples: Vec<f32>, at: u64) {
        let window = Window {
            samples,
            at,
            generation: self.generation.load(Ordering::Relaxed),
        };
        self.inbox.push(window);
    }

    /// Выбрасывает всё посчитанное и всё, что в пути: дорожку бросили, перемотали или сменили
    /// устройство, и прежние полосы к новому звуку отношения не имеют.
    pub fn clear(&self) {
        self.generation.fetch_add(1, Ordering::Release);
        self.inbox.clear();
    }
}

impl Drop for SpectrumThread {
    fn drop(&mut self) {
        self.inbox.close();
        if let Some(worker) = self.worker.take() {
            drop(worker.join());
        }
    }
}

/// Внутренности потока.
struct Spectrum {
    bands: Box<dyn BandsListener>,
    control: OutputControl,
    generation: Arc<AtomicU64>,
    /// Поколение, с которым поток работает сейчас.
    working: u64,
    /// Посчитанные полосы и показание счётчика, при котором их звук дойдёт до ушей.
    ahead: VecDeque<(u64, Vec<f64>)>,
}

impl Spectrum {
    fn new(
        bands: Box<dyn BandsListener>,
        control: OutputControl,
        generation: Arc<AtomicU64>,
    ) -> Self {
        Spectrum {
            bands,
            control,
            generation,
            working: 0,
            ahead: VecDeque::new(),
        }
    }

    fn serve(mut self, inbox: Arc<Inbox>) {
        loop {
            // Очередь пуста — ждать окно. Есть готовые к выдаче полосы — ждать не дольше срока
            // ближайших, но окно, придя раньше, разбудит само.
            let wait = match self.ahead.front() {
                None => Duration::from_millis(250),
                Some(_) => self.until_next(),
            };
            let waited = inbox.take(wait);

            let generation = self.generation.load(Ordering::Acquire);
            if generation != self.working {
                self.ahead.clear();
                self.working = generation;
            }

            match waited {
                None => return,
                Some(Some(window)) => {
                    // Окно от брошенной дорожки считать незачем.
                    if window.generation == self.working {
                        self.analyze(window);
                    }
                }
                Some(None) => {}
            }

            self.release();
        }
    }

    fn analyze(&mut self, window: Window) {
        let samples: Vec<f64> = window.samples.iter().map(|&sample| f64::from(sample)).collect();
        let bands = compute_bands(analyze_samples(&samples));
        self.ahead.push_back((window.at, bands));
    }

    /// Отдаёт те полосы, чей звук уже пошёл в уши.
    fn release(&mut self) {
        let heard = self.control.heard_samples();
        while let Some((at, _)) = self.ahead.front() {
            if *at > heard {
                break;
            }
            if let Some((_, bands)) = self.ahead.pop_front() {
                self.bands.on_bands(bands);
            }
        }
    }

    /// Сколько ждать до срока ближайших полос.
    fn until_next(&self) -> Duration {
        let at = match self.ahead.front() {
            Some((at, _)) => *at,
            None => return Duration::ZERO,
        };
        let left = at.saturating_sub(self.control.heard_samples());
        let spec = self.control.spec();
        let per_second = f64::from(spec.sample_rate) * f64::from(spec.channels);
        Duration::from_secs_f64(left as f64 / per_second)
    }
}
