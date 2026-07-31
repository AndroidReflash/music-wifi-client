//! Плеер снаружи: то, что зовёт Kotlin и примеры.
//!
//! Здесь же граница FFI, и она задаёт форму типов: номера дорожек — `u32`, а не
//! `usize` (на armeabi-v7a `usize` 32-битный, и одна и та же подпись означала бы на
//! двух ABI разное), время — миллисекунды `u64`, а не `Duration`. Внутрь эти доводы
//! не проходят: перевод делается прямо в теле метода.

use std::path::Path;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::error::PlayerError;
use super::worker::{lock, Command, Incoming, Status, Worker};
use crate::network::QuicSession;
use crate::player::decode::{DecodeThread, Progress};
use crate::player::decoder::cover_of;
use crate::player::file_scanner::TrackData;
use crate::player::output::{
    output_device_names, OutputControl, EQ_BANDS, EQ_FREQUENCIES, EQ_MAX_DB,
};
use crate::player::queue::RepeatMode;

/// Что плеер делает прямо сейчас.
#[derive(Debug, Clone, Copy, PartialEq, Eq, uniffi::Enum)]
pub enum PlayerState {
    /// Звука нет: ещё не начинали, остановили или кончилась очередь.
    Idle,
    Playing,
    Paused,
}

/// О чём плеер сообщает сам, не дожидаясь опроса.
///
/// Поля именованные: без имён uniffi зовёт их в Kotlin `v1`, и на той стороне пришлось
/// бы помнить, что означает первый довод.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum PlayerEvent {
    StateChanged { state: PlayerState },
    /// Пошла дорожка с таким номером в списке.
    TrackStarted { index: u32 },
    /// Дорожка доиграна до конца — включая то, что оставалось в кольце вывода.
    TrackFinished,
    /// Очередь кончилась: играть дальше нечего.
    QueueFinished,
    /// Дорожка не открылась или оборвалась на полпути.
    Failed { error: PlayerError },
    /// Позиция сменилась скачком, а не ходом воспроизведения: перемотка, пауза с выбросом
    /// кольца, смена устройства. Снимок к этому моменту уже пересчитан, его надо перечитать.
    ///
    /// Места здесь нет намеренно: позиция считается в момент опроса и живёт в снимке; ещё
    /// одно поле с ней означало бы два источника одной величины, которые разойдутся.
    PositionChanged,
}

/// Имена устройств вывода, на которые звук можно отправить.
///
/// Плеер для этого не нужен: спрашивают это, когда открывают настройки. В список попадают только
/// те устройства, на которых звук действительно заиграет, — см. `output_device_names`. Опрос стоит
/// времени, поэтому звать это в цикле отрисовки нельзя.
#[uniffi::export]
pub fn output_devices() -> Result<Vec<String>, PlayerError> {
    output_device_names().map_err(PlayerError::device)
}

/// Центры полос эквалайзера, Гц, от низкой к высокой. Их число — то, сколько усилений ждёт
/// [`Player::set_equalizer`]. Отдаются отсюда, чтобы подписи ползунков не расходились с тем,
/// что считает колбэк.
#[uniffi::export]
pub fn equalizer_bands() -> Vec<f32> {
    EQ_FREQUENCIES.to_vec()
}

/// Предел подъёма и ослабления полосы эквалайзера, дБ: ползунок ходит от минус этого числа до
/// плюс него. Больше плеер всё равно не возьмёт — лишнее обрезается.
#[uniffi::export]
pub fn equalizer_max_db() -> f32 {
    EQ_MAX_DB
}

/// Байты вложенной картинки дорожки — обложки. `None` означает, что её в файле нет.
///
/// Плеер для этого не нужен: обложку спрашивают и у той дорожки, которая не играет. Путь — к файлу
/// на этой машине; у дорожки с сервера обложки не будет, в протоколе такого запроса нет.
///
/// Байты отдаются как есть, в том виде, в каком лежали в файле. Разбирать их здесь нечем, а на
/// стороне Kotlin картинка распознаётся по содержимому, так что медиа-тип через границу не нужен.
#[uniffi::export]
pub fn track_cover(path: String) -> Result<Option<Vec<u8>>, PlayerError> {
    cover_of(Path::new(&path)).map_err(PlayerError::track)
}

/// Кому плеер рассказывает о событиях. Вызовы приходят из управляющего потока, и
/// задерживать их нельзя: пока получатель думает, команды не разбираются.
#[uniffi::export(callback_interface)]
pub trait PlayerListener: Send + Sync {
    fn on_event(&self, event: PlayerEvent);
}

/// Кому плеер отдаёт полосы спектра. Вызов приходит из потока декодирования, в тот миг,
/// когда кадры уходят устройству, — то есть ровно с темпом звука, без таймеров. Держать
/// этот вызов нельзя: пока получатель думает, кадры не идут.
#[uniffi::export(callback_interface)]
pub trait BandsListener: Send + Sync {
    fn on_bands(&self, bands: Vec<f64>);
}

/// То, что плеер отвечает на опрос.
#[derive(Debug, Clone, uniffi::Record)]
pub struct PlayerSnapshot {
    pub state: PlayerState,
    /// Сколько текущей дорожки уже прозвучало, в миллисекундах. Считается по тому, что
    /// устройство действительно забрало, а не по тому, что успел декодер.
    pub position_ms: u64,
    /// Длительность текущей дорожки в миллисекундах, если контейнер её объявил.
    pub duration_ms: Option<u64>,
    /// Номер текущей дорожки в списке — тот, который показывают пользователю.
    pub track_index: Option<u32>,
    pub track_count: u32,
    /// Есть ли куда идти кнопками. Уведомление на Android и MPRIS на десктопе
    /// спрашивают именно об этом, из списка они этого не выводят.
    pub has_next: bool,
    pub has_previous: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
}

/// Плеер. Сам он ничего не делает: владеет управляющим потоком и разговаривает с ним
/// командами, а обратно получает снимок состояния и события.
#[derive(uniffi::Object)]
pub struct Player {
    commands: Sender<Incoming>,
    status: Arc<Mutex<Status>>,
    progress: Arc<Mutex<Progress>>,
    output: OutputControl,
    /// Ручка управляющего потока, пока его не дождались. Замок здесь не ради гонки за
    /// ней, а ради `&self`: через FFI плеер живёт в `Arc`, своего `&mut self` не
    /// бывает ни у кого, а `shutdown` ручку забирает.
    worker: Mutex<Option<JoinHandle<()>>>,
}

#[uniffi::export]
impl Player {
    /// Открывает устройство вывода и поднимает оба потока — управляющий и декодирующий.
    ///
    /// `session` нужна только для дорожек с сервера; без неё играют локальные файлы, а
    /// сетевая дорожка честно скажет, что подключения нет.
    #[uniffi::constructor]
    pub fn new(
        listener: Box<dyn PlayerListener>,
        bands: Box<dyn BandsListener>,
        session: Option<Arc<QuicSession>>,
    ) -> Result<Arc<Self>, PlayerError> {
        // Приёмник журнала ставится до того, как кто-нибудь напишет первую строку:
        // фасад `log` без установленного приёмника выбрасывает записи молча. Место —
        // здесь, потому что плеер создаётся раньше любого звука, а новая точка входа
        // через границу FFI потребовала бы перегенерации биндингов.
        //
        // Только на Android: `init_once` занимает единственное место глобального
        // логгера, и на хосте он отнял бы его у `env_logger` в примере, печатая при
        // этом никуда. `init_once` терпит повторный вызов — второй плеер в том же
        // процессе ничего не сломает.
        let (commands, inbox) = mpsc::channel();
        // Устройство открывается здесь же: если его нет, узнать об этом надо сразу
        let decode = DecodeThread::start(session, commands.clone(), bands)?;
        let progress = decode.progress();
        let output = decode.control();

        let status = Arc::new(Mutex::new(Status {
            state: PlayerState::Idle,
            generation: 0,
            track_index: None,
            track_count: 0,
            has_next: false,
            has_previous: false,
            shuffle: false,
            repeat: RepeatMode::Off,
        }));

        let worker_status = status.clone();
        let worker = thread::Builder::new()
            .name("player".to_string())
            .spawn(move || Worker::new(decode, listener, worker_status).serve(inbox))
            .map_err(|err| PlayerError::Gone {
                reason: format!("управляющий поток не запустился: {err}"),
            })?;

        Ok(Arc::new(Player {
            commands,
            status,
            progress,
            output,
            worker: Mutex::new(Some(worker)),
        }))
    }

    /// Задаёт очередь. `start` — номер дорожки, с которой начинать; `None` означает
    /// «с первой». Воспроизведение не начинается: для этого есть `play`.
    ///
    /// Номер проверяется здесь, чтобы об ошибке узнал тот, кто её сделал: из потока
    /// она вернулась бы событием и неизвестно когда.
    pub fn set_queue(
        &self,
        tracks: Vec<TrackData>,
        start: Option<u32>,
    ) -> Result<(), PlayerError> {
        let start = start.map(|index| index as usize);
        if let Some(index) = start {
            if index >= tracks.len() {
                return Err(PlayerError::BadArgument {
                    reason: format!(
                        "начать просят с дорожки {}, а их всего {}",
                        index,
                        tracks.len()
                    ),
                });
            }
        }
        self.send(Command::SetQueue { tracks, start })
    }

    /// Играть: продолжить после паузы или начать текущую дорожку очереди.
    pub fn play(&self) -> Result<(), PlayerError> {
        self.send(Command::Play)
    }

    pub fn pause(&self) -> Result<(), PlayerError> {
        self.send(Command::Pause)
    }

    /// Играть дорожку с таким номером в списке.
    pub fn play_track(&self, index: u32) -> Result<(), PlayerError> {
        self.send(Command::PlayTrack(index as usize))
    }

    /// Перематывает текущую дорожку на такую миллисекунду. Без дорожки не делает ничего.
    pub fn seek(&self, position_ms: u64) -> Result<(), PlayerError> {
        self.send(Command::Seek(Duration::from_millis(position_ms)))
    }

    /// Следующая дорожка. На краю списка без повтора не делает ничего.
    pub fn next(&self) -> Result<(), PlayerError> {
        self.send(Command::Next)
    }

    pub fn previous(&self) -> Result<(), PlayerError> {
        self.send(Command::Previous)
    }

    /// Обрывает дорожку. Очередь и место в ней сохраняются: `play` начнёт ту же дорожку
    /// с начала.
    pub fn stop(&self) -> Result<(), PlayerError> {
        self.send(Command::Stop)
    }

    pub fn set_shuffle(&self, shuffle: bool) -> Result<(), PlayerError> {
        self.send(Command::SetShuffle(shuffle))
    }

    pub fn set_repeat(&self, repeat: RepeatMode) -> Result<(), PlayerError> {
        self.send(Command::SetRepeat(repeat))
    }

    /// Подключение к серверу для сетевых дорожек. Оно появляется позже плеера — адрес
    /// сервера вводят уже после запуска — и меняется вместе с адресом, поэтому задаётся
    /// отдельно от создания. `None` означает «сервера нет»: сетевая дорожка тогда честно
    /// скажет, что подключения нет.
    ///
    /// Играющую дорожку смена не трогает: она читает через то подключение, с которым
    /// открылась. Новое берёт следующая.
    pub fn set_session(&self, session: Option<Arc<QuicSession>>) -> Result<(), PlayerError> {
        self.send(Command::SetSession(session))
    }

    /// Громкость приложения, от 0 до 1. Не системная: умножается то, что мы сами отдаём
    /// устройству.
    ///
    /// Приказом не идёт и очереди не ждёт — это один атомарный признак, который читает колбэк
    /// устройства. Поэтому громкость меняется сразу, в том числе пока поток декодирования стоит
    /// на сетевом чтении.
    pub fn set_volume(&self, volume: f32) {
        self.output.set_volume(volume);
    }

    /// Усиления полос эквалайзера, дБ, по одному на каждую из [`equalizer_bands`], в том же
    /// порядке. Действует сразу, по той же причине, что и громкость: приказом не идёт, колбэк
    /// устройства сам сверяет настройки на каждом периоде.
    ///
    /// Звук ослабляется на самый большой подъём, чтобы поднятая полоса не выводила его за
    /// предел устройства: подъём одних полос на слух — ослабление остальных.
    pub fn set_equalizer(&self, gains_db: Vec<f32>) -> Result<(), PlayerError> {
        let gains_db: [f32; EQ_BANDS] =
            gains_db
                .try_into()
                .map_err(|given: Vec<f32>| PlayerError::BadArgument {
                    reason: format!(
                        "у эквалайзера {EQ_BANDS} полос, а усилений пришло {}",
                        given.len()
                    ),
                })?;
        self.output.set_equalizer(gains_db);
        Ok(())
    }

    /// Куда выводить звук: имя из [`output_devices`] или `None` — то устройство, которое система
    /// считает основным.
    ///
    /// Дорожка при этом продолжает играть. Пропадёт только то, что лежало в кольце прежнего
    /// устройства, — на слух это прыжок вперёд до длины кольца. Если устройство не открылось,
    /// звук остаётся на прежнем, а наружу приходит [`PlayerEvent::Failed`].
    pub fn set_output_device(&self, device: Option<String>) -> Result<(), PlayerError> {
        self.send(Command::SetOutputDevice(device))
    }

    /// Снимок состояния. Позиция считается здесь и сейчас — по счётчику устройства,
    /// поэтому она свежая и между событиями.
    pub fn snapshot(&self) -> PlayerSnapshot {
        let status = *lock(&self.status);
        let progress = *lock(&self.progress);

        // Прогресс от дорожки, которую уже бросили, к нынешнему состоянию отношения не
        // имеет: поколения разойдутся, и позиция будет нулевой.
        let playing_now = progress.generation != 0 && progress.generation == status.generation;
        let (position, duration) = if playing_now {
            let position = progress.position(self.output.delivered_samples(), self.output.spec());
            (position, progress.duration)
        } else {
            (Duration::ZERO, None)
        };

        PlayerSnapshot {
            state: status.state,
            position_ms: millis(position),
            duration_ms: duration.map(millis),
            track_index: status.track_index.map(|index| index as u32),
            track_count: status.track_count as u32,
            has_next: status.has_next,
            has_previous: status.has_previous,
            shuffle: status.shuffle,
            repeat: status.repeat,
        }
    }

    /// Останавливает потоки и дожидается их. То же делает `drop`, но молча — здесь о
    /// неудаче можно узнать. Второй вызов ответит `Gone`: плеера к тому времени и
    /// правда нет.
    pub fn shutdown(&self) -> Result<(), PlayerError> {
        self.commands
            .send(Incoming::Command(Command::Shutdown))
            .map_err(|_| PlayerError::gone("управляющий поток уже не отвечает"))?;
        let worker = lock(&self.worker).take();
        match worker {
            Some(worker) => worker
                .join()
                .map_err(|_| PlayerError::gone("управляющий поток упал с паникой")),
            None => Ok(()),
        }
    }
}

impl Player {
    fn send(&self, command: Command) -> Result<(), PlayerError> {
        self.commands
            .send(Incoming::Command(command))
            .map_err(|_| PlayerError::gone("управляющий поток не отвечает"))
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        // Неудача отправки означает, что поток уже кончился, — тогда и будить некого,
        // и ждать нечего. Рассказать о ней в `drop` всё равно некому: для этого есть
        // `shutdown`.
        if self
            .commands
            .send(Incoming::Command(Command::Shutdown))
            .is_ok()
        {
            if let Some(worker) = lock(&self.worker).take() {
                drop(worker.join());
            }
        }
    }
}

/// Время для той стороны границы: миллисекунды, потому что `java.time.Duration` в
/// Kotlin есть не на всяком Android, а целое число миллисекунд есть везде.
fn millis(duration: Duration) -> u64 {
    duration.as_millis() as u64
}
