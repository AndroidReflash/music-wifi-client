//! Поток спектра: преобразование Фурье и отдача полос наружу.
//!
//! Зачем отдельный поток. Полосы считаются из тех же кадров, что уходят устройству, и раньше
//! считались там же, где эти кадры готовятся, — в потоке декодирования. У того потока одна
//! обязанность: успевать кормить кольцо вывода, иначе устройство получит тишину и это слышно.
//! Преобразование Фурье и вызов через границу в Kotlin (а это JNI, десятки микросекунд) к этой
//! обязанности отношения не имеют и только мешали ей.

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use log::error;

use crate::player::audioconverter::compute_bands;
use crate::player::fft_processor::analyze_samples;
use crate::player::output::OutputControl;
use crate::player::player_core::BandsListener;

/// Окно отсчётов, из которого считать полосы.
struct Window {
    samples: Vec<f32>,
    /// Показание счётчика отданных отсчётов, при котором эти полосы зазвучат.
    at: u64,
    /// Поколение: окно от дорожки, которую уже бросили, считать незачем.
    generation: u64,
}

/// Ручка потока спектра у того, кто его кормит.
pub struct SpectrumThread {
    windows: Sender<Window>,
    generation: Arc<AtomicU64>,
    worker: Option<JoinHandle<()>>,
}

impl SpectrumThread {
    pub fn start(bands: Box<dyn BandsListener>, control: OutputControl) -> Self {
        let (windows, inbox) = channel();
        let generation = Arc::new(AtomicU64::new(0));
        let worker_generation = generation.clone();

        // Поток спектра не обязателен для звука, поэтому его неудача не обрывает создание плеера.
        // Но и молчать о ней нельзя: без него визуализатор просто не появится, и это надо будет
        // откуда-то узнать.
        let worker = match thread::Builder::new()
            .name("spectrum".to_string())
            .spawn(move || Spectrum::new(bands, control, worker_generation).serve(inbox))
        {
            Ok(worker) => Some(worker),
            Err(err) => {
                error!("поток спектра не запустился, визуализатора не будет: {err}");
                None
            }
        };

        SpectrumThread {
            windows,
            generation,
            worker,
        }
    }

    /// Отдаёт окно на счёт. Канал без предела, поэтому отправка не ждёт никогда и окно не
    /// теряется: отстал поток спектра — окна дождутся его в канале.
    pub fn feed(&self, samples: Vec<f32>, at: u64) {
        let window = Window {
            samples,
            at,
            generation: self.generation.load(Ordering::Relaxed),
        };
        // Отказ бывает только тогда, когда потока спектра нет: он не запустился — об этом
        // сказано при старте — или кончился. Окну тогда деваться некуда.
        drop(self.windows.send(window));
    }

    /// Выбрасывает всё посчитанное и всё, что в пути: дорожку бросили, перемотали или сменили
    /// устройство, и прежние полосы к новому звуку отношения не имеют.
    ///
    /// Сделано номером поколения, а не сообщением: сообщение пришлось бы слать по тому же каналу,
    /// а он переполняется и теряет — и просьба очистить пропала бы вместе с окном.
    pub fn clear(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

impl Drop for SpectrumThread {
    fn drop(&mut self) {
        // Закрытие канала — это и есть приказ уйти: поток ждёт на нём и выходит по разрыву.
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

    fn serve(mut self, inbox: Receiver<Window>) {
        loop {
            // Очередь пуста — ждать окно и только его. Есть очередь — ждать до срока ближайших
            // полос, но не дольше: окно, придя раньше, разбудит само.
            let waited = match self.ahead.front() {
                None => inbox.recv().map_err(|_| RecvTimeoutError::Disconnected),
                Some(_) => inbox.recv_timeout(self.until_next()),
            };

            // Сброс сверяется после ожидания, а не до: он мог прийти, пока поток ждал, и тогда
            // окно, которое его разбудило, уже от нового поколения.
            let generation = self.generation.load(Ordering::Relaxed);
            if generation != self.working {
                self.ahead.clear();
                self.working = generation;
            }

            match waited {
                Ok(window) => {
                    // Окно от брошенной дорожки считать незачем.
                    if window.generation == self.working {
                        self.analyze(window);
                    }
                }
                Err(RecvTimeoutError::Timeout) => {}
                Err(RecvTimeoutError::Disconnected) => return,
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
