// Управляющий поток: очередь, состояние, события. Ничего, что может ждать.

use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use super::api::{PlayerEvent, PlayerListener, PlayerState};
use super::error::PlayerError;
use crate::network::QuicSession;
use crate::player::decode::{DecodeThread, Order, Report};
use crate::player::file_scanner::TrackData;
use crate::player::output::OutputControl;
use crate::player::queue::{Queue, RepeatMode};
use crate::player::quic_source::TrackCancel;

/// То, чем распоряжается управляющий поток. Позиции здесь нет: она складывается из
/// счётчика устройства и места дорожки, а место знает поток декодирования.
#[derive(Clone, Copy)]
pub(super) struct Status {
    pub(super) state: PlayerState,
    /// Поколение дорожки, которую играют сейчас. Ноль — дорожки нет.
    pub(super) generation: u64,
    pub(super) track_index: Option<usize>,
    pub(super) track_count: usize,
    pub(super) has_next: bool,
    pub(super) has_previous: bool,
    pub(super) shuffle: bool,
    pub(super) repeat: RepeatMode,
}

/// Что приходит в ящик управляющего потока. Ящик один на команды и сообщения: ждать на
/// двух очередях сразу не умеет никто.
pub enum Incoming {
    Command(Command),
    Report(Report),
}

pub enum Command {
    SetQueue {
        tracks: Vec<TrackData>,
        start: Option<usize>,
    },
    Play,
    /// Дорожка по номеру в списке — нажатие на песню на экране.
    PlayTrack(usize),
    Pause,
    Next,
    Previous,
    Seek(Duration),
    Stop,
    SetShuffle(bool),
    SetRepeat(RepeatMode),
    /// Подключение к серверу для сетевых дорожек.
    SetSession(Option<Arc<QuicSession>>),
    /// Устройство вывода по имени; `None` — основное в системе.
    SetOutputDevice(Option<String>),
    Shutdown,
}

/// Управляющий поток: очередь, состояние, события. Ничего, что может ждать.
pub(super) struct Worker {
    decode: DecodeThread,
    listener: Box<dyn PlayerListener>,
    status: Arc<Mutex<Status>>,
    state: PlayerState,
    queue: Queue,
    /// Отмена дорожки, которая сейчас у потока декодирования. Ею обрывается его
    /// ожидание — без неё смена дорожки ждала бы сервера.
    cancel: Option<TrackCancel>,
    /// Сквозной счётчик дорожек: каждая новая получает следующий номер. Номера не
    /// возвращаются — иначе сообщение о брошенной дорожке попало бы в новую с тем же
    /// номером.
    generation: u64,
    /// Номер дорожки, которая сейчас у потока декодирования.
    current: Option<u64>,
    /// Рычаги устройства: пауза и счётчик отданного.
    output: OutputControl,
}

impl Worker {
    pub(super) fn new(
        decode: DecodeThread,
        listener: Box<dyn PlayerListener>,
        status: Arc<Mutex<Status>>,
    ) -> Self {
        Worker {
            output: decode.control(),
            decode,
            listener,
            status,
            state: PlayerState::Idle,
            queue: Queue::new(),
            cancel: None,
            generation: 0,
            current: None,
        }
    }

    pub(super) fn serve(mut self, inbox: Receiver<Incoming>) {
        loop {
            match inbox.recv() {
                Ok(Incoming::Command(Command::Shutdown)) => break,
                Ok(Incoming::Command(command)) => self.apply(command),
                Ok(Incoming::Report(report)) => self.handle(report),
                // Ящик закрылся: и хозяин, и поток декодирования ушли.
                Err(_) => break,
            }
            self.publish();
        }

        // Ждущую дорожку надо оборвать: поток декодирования, стоящий на сетевом чтении,
        // до приказа выключиться не дойдёт.
        self.drop_track();
        drop(self.decode.shutdown());
    }

    fn apply(&mut self, command: Command) {
        match command {
            Command::SetQueue { tracks, start } => {
                self.drop_track();
                self.set_state(PlayerState::Idle);
                if let Err(err) = self.queue.set_tracks(tracks, start) {
                    self.fail(PlayerError::bad_argument(err));
                }
            }
            Command::Play => self.play(),
            Command::PlayTrack(index) => match self.queue.jump_to(index) {
                Ok(()) => self.start_current(),
                Err(err) => self.fail(PlayerError::bad_argument(err)),
            },
            Command::Pause => {
                if self.state == PlayerState::Playing {
                    self.output.set_playing(false);
                    drop(self.decode.order(Order::Pause(true)));
                    self.set_state(PlayerState::Paused);
                }
            }
            Command::Next => self.step(true),
            Command::Previous => self.step(false),
            Command::Seek(position) => self.seek(position),
            Command::Stop => {
                self.drop_track();
                self.set_state(PlayerState::Idle);
            }
            Command::SetOutputDevice(device) => {
                if let Err(err) = self.decode.order(Order::SetOutputDevice(device)) {
                    self.fail(err);
                }
            }
            Command::SetSession(session) => {
                if let Err(err) = self.decode.order(Order::SetSession(session)) {
                    self.fail(err);
                }
            }
            Command::SetShuffle(shuffle) => self.queue.set_shuffle(shuffle),
            Command::SetRepeat(repeat) => self.queue.set_repeat(repeat),
            // Выключение разбирается в `serve`: оттуда выходят, а не продолжают.
            Command::Shutdown => {}
        }
    }

    /// Сообщение от потока декодирования. О дорожке, которую уже бросили, говорить
    /// нечего: её обрывали сами, и ошибка отсюда — след этого обрыва.
    fn handle(&mut self, report: Report) {
        match report {
            Report::Started(generation) if Some(generation) == self.current => {
                // Кольцо наполняется новой дорожкой — можно снимать тишину.
                if self.state == PlayerState::Playing {
                    self.output.set_playing(true);
                }
                if let Some(index) = self.queue.current_index() {
                    self.listener.on_event(PlayerEvent::TrackStarted {
                        index: index as u32,
                    });
                }
            }
            Report::Finished(generation) if Some(generation) == self.current => {
                self.current = None;
                self.cancel = None;
                self.listener.on_event(PlayerEvent::TrackFinished);

                // Список замкнут: с последней дорожки идём на первую. Ветка ниже остаётся
                // для пустого списка — на непустом `advance` дорожку находит всегда, и
                // событие о конце очереди наружу больше не уходит.
                if self.queue.advance().is_some() {
                    self.start_current();
                } else {
                    self.output.set_playing(false);
                    self.set_state(PlayerState::Idle);
                    self.listener.on_event(PlayerEvent::QueueFinished);
                }
            }
            // Устройство не открылось. Дорожку не трогаем: она играет на прежнем устройстве,
            // и единственное, что надо сделать, — сказать человеку.
            Report::DeviceFailed(error) => {
                self.listener.on_event(PlayerEvent::Failed { error });
            }
            // Позиция ушла скачком. Опрос из Kotlin идёт только пока играет, и после
            // паузы он остановится, поэтому без этого сообщения показанное время осталось
            // бы прежним — а оно уже другое.
            Report::PositionChanged(generation) if Some(generation) == self.current => {
                self.listener.on_event(PlayerEvent::PositionChanged);
            }
            // Устройство сменилось на ходу. Поток декодирования уже переоткрыл его и
            // замолчал; здесь остаётся признать паузу и сказать о ней наружу, иначе на
            // экране осталось бы «играет», а звука нет.
            Report::OutputChanged(generation) if Some(generation) == self.current => {
                if self.state == PlayerState::Playing {
                    self.set_state(PlayerState::Paused);
                }
            }
            Report::Failed(generation, error) if Some(generation) == self.current => {
                self.drop_track();
                self.set_state(PlayerState::Idle);
                self.listener.on_event(PlayerEvent::Failed { error });
            }
            _ => {}
        }
    }

    fn play(&mut self) {
        match self.state {
            PlayerState::Playing => {}
            PlayerState::Paused => {
                self.output.set_playing(true);
                drop(self.decode.order(Order::Pause(false)));
                self.set_state(PlayerState::Playing);
            }
            PlayerState::Idle => {
                if self.queue.current().is_some() {
                    self.start_current();
                } else {
                    self.listener.on_event(PlayerEvent::Failed {
                        error: PlayerError::BadArgument {
                            reason: "очередь пуста".to_string(),
                        },
                    });
                }
            }
        }
    }

    /// Шаг кнопкой. Список замкнут, поэтому с последней дорожки шаг ведёт на первую, а с
    /// первой назад — на последнюю; не делает ничего только на пустом списке.
    fn step(&mut self, forward: bool) {
        let moved = if forward {
            self.queue.next().is_some()
        } else {
            self.queue.previous().is_some()
        };
        if moved {
            self.start_current();
        }
    }

    fn seek(&mut self, position: Duration) {
        let Some(generation) = self.current else {
            return;
        };
        let order = Order::Seek {
            position,
            generation,
        };
        if let Err(err) = self.decode.order(order) {
            self.fail(err);
        }
    }

    /// Отдаёт текущую дорожку очереди потоку декодирования. Возврата не ждёт: пока
    /// дорожка открывается, команды разбираются как обычно.
    fn start_current(&mut self) {
        let Some(track) = self.queue.current().cloned() else {
            self.drop_track();
            self.set_state(PlayerState::Idle);
            return;
        };

        self.cancel_track();
        self.generation += 1;
        self.current = Some(self.generation);
        let cancel = TrackCancel::new();
        self.cancel = Some(cancel.clone());

        // До первых кадров новой дорожки устройство молчит: в кольце ещё лежит прежняя,
        // и без этого её остаток прозвучал бы поверх.
        self.output.set_playing(false);
        self.set_state(PlayerState::Playing);

        let order = Order::Open {
            track,
            cancel,
            generation: self.generation,
        };
        if let Err(err) = self.decode.order(order) {
            self.fail(err);
        }
    }

    /// Бросает дорожку: обрывает ожидание, глушит устройство и велит потоку
    /// декодирования забыть её. Состояние остаётся на усмотрение вызывающего.
    fn drop_track(&mut self) {
        self.cancel_track();
        self.current = None;
        self.output.set_playing(false);
        drop(self.decode.order(Order::Drop));
    }

    fn cancel_track(&mut self) {
        if let Some(cancel) = self.cancel.take() {
            cancel.cancel();
        }
    }

    fn fail(&mut self, err: PlayerError) {
        self.drop_track();
        self.set_state(PlayerState::Idle);
        self.listener.on_event(PlayerEvent::Failed { error: err });
    }

    fn set_state(&mut self, state: PlayerState) {
        if self.state == state {
            return;
        }
        self.state = state;
        self.listener.on_event(PlayerEvent::StateChanged { state });
    }

    /// Обновляет снимок, по которому плеер отвечает на опрос.
    fn publish(&mut self) {
        let updated = Status {
            state: self.state,
            generation: self.current.unwrap_or(0),
            track_index: self.queue.current_index(),
            track_count: self.queue.len(),
            has_next: self.queue.has_next(),
            has_previous: self.queue.has_previous(),
            shuffle: self.queue.shuffle(),
            repeat: self.queue.repeat(),
        };
        *lock(&self.status) = updated;
    }
}

/// Замок, который не роняет плеер из-за чужой паники: под ним лежит снимок, и он цел.
pub(super) fn lock<T>(what: &Mutex<T>) -> MutexGuard<'_, T> {
    what.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
