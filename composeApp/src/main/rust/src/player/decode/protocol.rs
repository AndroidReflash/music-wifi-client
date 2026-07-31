//! Договор между управляющим потоком и потоком декодирования: приказы туда, сообщения
//! оттуда, место в дорожке для тех, кто спрашивает позицию, — и ручка самого потока.

use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;
use super::thread::Decode;
use crate::network::QuicSession;
use crate::player::decoder::PcmSpec;
use crate::player::file_scanner::TrackData;
use crate::player::output::OutputControl;
use crate::player::player_core::{BandsListener, Incoming, PlayerError};
use crate::player::quic_source::TrackCancel;

/// Приказ потоку декодирования.
pub enum Order {
    /// Бросить прежнюю дорожку, очистить кольцо и открыть эту.
    Open {
        track: TrackData,
        /// Отмена этой дорожки. Её же держит управляющий поток: пока здесь ждут
        /// байтов, оборвать ожидание можно только оттуда.
        cancel: TrackCancel,
        generation: u64,
    },
    Seek {
        position: Duration,
        generation: u64,
    },
    /// Пауза или продолжение. Звук глушится не этим, а признаком устройства: пауза должна
    /// срабатывать и тогда, когда этот поток стоит на сетевом чтении.
    ///
    /// Здесь — всё остальное. На паузе поток выбрасывает кольцо вместе с очередью полос и
    /// засыпает; продолжение наполняет их заново, как на старте дорожки.
    Pause(bool),
    /// Бросить дорожку и очистить кольцо.
    Drop,
    /// Перейти на другое устройство вывода; `None` — на основное в системе. Дорожка при этом
    /// продолжает играть: смену сделали руками, обрывать её незачем.
    SetOutputDevice(Option<String>),
    /// Подключение к серверу: оно появляется позже плеера и меняется вместе с адресом,
    /// поэтому передаётся приказом, а не только при создании. Уже открытая дорожка
    /// держит прежнее подключение — новое берёт следующая.
    SetSession(Option<Arc<QuicSession>>),
    Shutdown,
}

/// Сообщение от потока декодирования. Номер поколения в каждом — чтобы управляющий
/// поток узнал дорожку, о которой речь: пока сообщение шло, дорожка могла смениться.
pub enum Report {
    /// Дорожка открылась, дальше кольцо наполняется её кадрами.
    Started(u64),
    /// Дорожка доиграна до конца — включая то, что оставалось в кольце.
    Finished(u64),
    /// Дорожка не открылась или оборвалась на полпути.
    Failed(u64, PlayerError),
    /// Место в дорожке пересчитано скачком: перемотка или выброс кольца на паузе. Снимок к
    /// этому моменту уже несёт новое место.
    PositionChanged(u64),
    /// Устройство вывода не открылось. К дорожке это отношения не имеет — она играет на прежнем,
    /// — поэтому и номера поколения здесь нет: сказать надо человеку, а не бросать дорожку.
    DeviceFailed(PlayerError),
    /// Устройство пропало, и звук уехал на то, которое система считает основным теперь.
    /// Дорожка при этом жива и стоит на паузе: снятые наушники не означают музыку из
    /// динамика на весь автобус.
    OutputChanged(u64),
}

/// Место в дорожке, от которого считается позиция. Пишет этот поток, читает тот, кто
/// спрашивает снимок.
#[derive(Clone, Copy)]
pub struct Progress {
    /// Поколение дорожки, к которой всё это относится. Ноль означает, что дорожки нет.
    pub generation: u64,
    /// Место дорожки, которому отвечает `delivered_at_start`: ноль с начала дорожки и
    /// место перемотки после неё.
    pub offset: Duration,
    /// Показание счётчика устройства на этом месте.
    pub delivered_at_start: u64,
    pub duration: Option<Duration>,
}

impl Progress {
    /// Место в дорожке на сейчас: отложенное плюс то, что устройство успело сыграть с
    /// этого места. Считает это и снимок наружу, и смена устройства, — поэтому счёт
    /// живёт здесь, а не в двух копиях, которые разойдутся.
    ///
    /// `spec` — того устройства, которое играло эти отсчёты: частота у нового может быть
    /// другой, и считать по ней чужие отсчёты нельзя.
    pub fn position(&self, delivered_now: u64, spec: PcmSpec) -> Duration {
        let delivered = delivered_now.saturating_sub(self.delivered_at_start);
        let frames = delivered / u64::from(spec.channels);
        self.offset + Duration::from_secs_f64(frames as f64 / f64::from(spec.sample_rate))
    }

    pub(super) fn none() -> Self {
        Progress {
            generation: 0,
            offset: Duration::ZERO,
            delivered_at_start: 0,
            duration: None,
        }
    }
}

/// Ручка потока декодирования у того, кто им командует.
pub struct DecodeThread {
    orders: Sender<Order>,
    progress: Arc<Mutex<Progress>>,
    control: OutputControl,
    worker: Option<JoinHandle<()>>,
}

impl DecodeThread {
    /// Поднимает поток и дожидается, пока тот откроет устройство: об отказе устройства
    /// надо узнать здесь и сейчас, а не сообщением неизвестно когда.
    ///
    /// `reports` — тот же ящик, в который шлют команды: у управляющего потока он один,
    /// иначе ему пришлось бы ждать на двух очередях сразу, а так не умеет никто.
    pub fn start(
        session: Option<Arc<QuicSession>>,
        reports: Sender<Incoming>,
        bands: Box<dyn BandsListener>,
    ) -> Result<Self, PlayerError> {
        let (orders, inbox) = mpsc::channel();
        let (ready, started) = mpsc::channel();
        let progress = Arc::new(Mutex::new(Progress::none()));
        let worker_progress = progress.clone();
        let worker = thread::Builder::new()
            .name("decode".to_string())
            .spawn(move || Decode::run(inbox, reports, worker_progress, session, bands, ready))
            .map_err(|err| {
                    PlayerError::Gone {
                        reason: format!("поток декодирования не запустился: {err}"),
                    }
                })?;
        // Устройство открывается внутри потока: поток cpal привязан к тому, кто его
        // создал, и передать его потом нельзя. Значит, об успехе можно узнать только
        // ответом.
        match started.recv() {
            Ok(Ok(control)) => Ok(DecodeThread {
                orders,
                progress,
                control,
                worker: Some(worker),
            }),
            Ok(Err(err)) => {
                Err(err)
            },
            Err(_) => {
                Err(PlayerError::gone("поток декодирования умер, не сказав ни слова"))
            },
        }
    }

    pub fn progress(&self) -> Arc<Mutex<Progress>> {
        self.progress.clone()
    }

    pub fn control(&self) -> OutputControl {
        self.control.clone()
    }

    /// Отдаёт приказ. Неудача означает, что поток кончился.
    pub fn order(&self, order: Order) -> Result<(), PlayerError> {
        self.orders
            .send(order)
            .map_err(|_| PlayerError::gone("поток декодирования не отвечает"))
    }

    /// Останавливает поток и дожидается его.
    ///
    /// Дорожку перед этим надо оборвать: поток, стоящий на сетевом чтении, до приказа
    /// выключиться не дойдёт, и ожидание здесь не кончится никогда.
    pub fn shutdown(mut self) -> Result<(), PlayerError> {
        self.order(Order::Shutdown)?;
        match self.worker.take() {
            Some(worker) => worker
                .join()
                .map_err(|_| PlayerError::gone("поток декодирования упал с паникой")),
            None => Ok(()),
        }
    }
}

impl Drop for DecodeThread {
    fn drop(&mut self) {
        if self.orders.send(Order::Shutdown).is_ok() {
            if let Some(worker) = self.worker.take() {
                drop(worker.join());
            }
        }
    }
}
