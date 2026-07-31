use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use log::{debug, error, info, trace, warn};
use anyhow::{anyhow, Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{
    BufferSize, Device, Host, OutputCallbackInfo, SampleFormat, Stream, StreamConfig,
    SupportedBufferSize,
};
use rtrb::{Producer, RingBuffer};

use crate::player::decoder::PcmSpec;

use super::equalizer::{EqSettings, Equalizer, EQ_BANDS};

/// Длина буфера устройства в кадрах. 2048 кадров — это 46 мс при 44100 Гц: запаса
/// хватает, чтобы занятая система не успела оборвать звук, а для музыки такая задержка
/// ничего не значит. Оставлять выбор устройству нельзя: оно берёт разное от запуска к
/// запуску, и поведение перестаёт быть воспроизводимым.
///
/// Колбэк приходит чаще, чем раз в буфер: ALSA внутри cpal делит его на периоды по
/// четверти длины и зовёт на каждый период.
pub const DEVICE_BUFFER_FRAMES: u32 = 2048;

/// Во сколько раз кольцо между декодером и устройством длиннее буфера устройства.
/// Восемь буферов — это около трети секунды: настолько декодер может отстать, не прервав
/// звук.
const RING_BUFFERS: usize = 8;

/// Объявленный устройством минимум длины буфера, приведённый к кадрам.
///
/// На Android в этом поле лежат не кадры, а байты. cpal 0.15.3 берёт минимум у
/// Java-метода `AudioTrack.getMinBufferSize`, который отвечает в байтах
/// (`src/host/oboe/mod.rs`, `buffer_size_range_for_params`), и кладёт ответ как есть в
/// поле, которое по его же документации измеряется в кадрах. Дальше это число уходит в
/// `set_buffer_capacity_in_frames`, и буфер получается во столько раз длиннее нужного,
/// сколько в кадре байтов: на стерео `f32` — в восемь. Замерено 2026-09-21 на телефоне:
/// динамик объявил 15104 (это 1888 кадров, 43 мс), а устройство получило просьбу на 15104
/// кадра, то есть 342 мс; Bluetooth объявил 54000 (6750 кадров, 153 мс) и получил 1224 мс.
/// Отсюда и задержка паузы: она не может обогнать то, что уже отдано драйверу.
///
/// На хосте (ALSA) в поле действительно кадры, и делить нечего.
#[cfg(target_os = "android")]
fn declared_min_frames(min: u32, bytes_per_frame: u32) -> u32 {
    min / bytes_per_frame
}

#[cfg(not(target_os = "android"))]
fn declared_min_frames(min: u32, _bytes_per_frame: u32) -> u32 {
    min
}

/// Звуковой выход: поток cpal и кольцо, через которое ему отдают кадры.
///
/// Колбэк устройства живёт в реальном времени — в нём нельзя ни выделять память, ни ждать
/// на замке, поэтому единственное, что он делает, это забирает готовые отсчёты из кольца.
pub struct AudioOutput {
    /// Живой поток. Его нельзя ронять на ходу — см. `drain_and_stop`.
    stream: Stream,
    producer: Producer<f32>,
    /// Запущен ли поток. Пока в кольце нет звука, запускать его нечем.
    started: bool,
    /// Сколько отсчётов должно накопиться в кольце до запуска устройства — один его буфер.
    prefill: usize,
    buffer_frames: u32,
    device_name: String,
    /// Сколько играет один буфер устройства. Раньше этого срока место в кольце не
    /// освободится, и на этом строится ожидание.
    period: Duration,
    shared: Arc<Shared>,
}

/// То, что живёт дольше потока cpal.
///
/// Поток, кольцо и длина буфера при смене устройства создаются заново, а это — нет:
/// сюда смотрит `OutputControl` из чужого потока, а получен он один раз, при создании
/// плеера. Создай признаки и счётчики заново — и его рычаги вели бы к мёртвому потоку,
/// то есть пауза перестала бы работать, а позиция замерла бы навсегда.
struct Shared {
    /// Играть или молчать. Читается колбэком на каждом периоде, поэтому пауза
    /// срабатывает в пределах одного периода, а не одного кольца.
    playing: AtomicBool,
    /// Просьба выбросить всё, что лежит в кольце. Выполняет её колбэк: только он владеет
    /// читающей стороной.
    discard: AtomicBool,
    /// Сколько отсчётов колбэк отдал тишиной, не дождавшись данных.
    starved: AtomicU64,
    /// Сколько раз устройство вообще позвало колбэк. Ноль означает, что поток не идёт,
    /// и тогда искать причину надо в cpal, а не в кадрах.
    callbacks: AtomicU64,
    /// Сколько отсчётов устройство забрало из кольца. При смене устройства счёт
    /// продолжается: по нему считается позиция, а дорожка от смены устройства не меняется.
    delivered: AtomicU64,
    /// Точка отсчёта для `called_at`: атомарного `Instant` нет, поэтому момент хранится
    /// наносекундами от неё.
    epoch: Instant,
    /// Когда колбэк нынешнего потока звали последний раз, наносекундами от `epoch`. Ноль —
    /// у этого потока колбэка ещё не было. От этого момента досчитывается счётчик отданного
    /// между колбэками, см. `OutputControl::heard_samples`.
    called_at: AtomicU64,
    /// Устройство пропало — например, отключили наушники. Ставит колбэк ошибок, снимает
    /// переоткрытие. Пока признак стоит, поток cpal мёртв: колбэк его больше не зовут,
    /// кольцо не пустеет, и кадры девать некуда.
    failed: AtomicBool,
    errors: Mutex<Vec<String>>,
    /// Частота и каналы нынешнего устройства. Не копия у владельца, а общее место:
    /// по ним считается позиция, а при смене устройства они меняются.
    spec: Mutex<PcmSpec>,
    /// Выбранное устройство по имени; `None` — то, которое система считает основным. Лежит
    /// здесь, потому что переоткрытие обязано открывать заново именно выбранное.
    chosen: Mutex<Option<String>>,
    /// Множитель громкости, битами `f32`: колбэк живёт в реальном времени, и замок ему брать
    /// нельзя, а атомарного `f32` в языке нет.
    gain: AtomicU32,
    /// Усиления полос эквалайзера. Лежат здесь, а не в колбэке, по той же причине, что и
    /// громкость: колбэк при смене устройства создаётся заново, а выбор человека остаётся.
    eq: EqSettings,
}

impl Shared {
    fn new(spec: PcmSpec, chosen: Option<String>) -> Self {
        Shared {
            chosen: Mutex::new(chosen),
            playing: AtomicBool::new(true),
            discard: AtomicBool::new(false),
            starved: AtomicU64::new(0),
            callbacks: AtomicU64::new(0),
            delivered: AtomicU64::new(0),
            epoch: Instant::now(),
            called_at: AtomicU64::new(0),
            failed: AtomicBool::new(false),
            errors: Mutex::new(Vec::new()),
            spec: Mutex::new(spec),
            gain: AtomicU32::new(1.0f32.to_bits()),
            eq: EqSettings::new(),
        }
    }

    /// Отмечает вызов колбэка. Колбэк зовёт это последним делом, после счётчика отданного:
    /// читающий берёт момент первым и потому не увидит момент новее счётчика.
    ///
    /// Часы здесь можно: монотонные часы читаются без замка и без выделения памяти.
    fn mark_called(&self) {
        let nanos = self.epoch.elapsed().as_nanos() as u64;
        self.called_at.store(nanos, Ordering::Release);
    }
}

/// Имена устройств вывода, на которых звук действительно заиграет.
///
/// Отбираются те, кто ответил на `default_output_config()` и принимает `f32`: вывод написан
/// только под него, и показывать то, на чём открытие кончится ошибкой, незачем. На Linux ALSA
/// отдаёт вперемешку общие входы (`default`, `pulse`, `pipewire`) и сырые устройства
/// (`hw:CARD=…`, `front:CARD=…`); этот отбор — единственное, что отличает список для человека от
/// списка для отладки, и он же убирает сырые устройства, занятые сервером звука: на опросе они
/// не открываются.
///
/// Опрос каждого устройства стоит времени — на Android это вызовы через JNI, — поэтому звать это
/// надо тогда, когда открывают настройки, а не по ходу воспроизведения.
pub fn output_device_names() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .context("система не отдала список устройств вывода")?;

    Ok(devices
        .filter_map(|device| {
            let config = device.default_output_config().ok()?;
            if config.sample_format() != SampleFormat::F32 {
                return None;
            }
            device.name().ok()
        })
        .collect())
}

/// Устройство с таким именем среди устройств вывода.
fn find_output_device(host: &Host, name: &str) -> Option<Device> {
    host.output_devices()
        .ok()?
        .find(|device| device.name().map(|it| it == name).unwrap_or(false))
}

/// Выбранное устройство из общего состояния.
fn chosen_of(shared: &Shared) -> Option<String> {
    match shared.chosen.lock() {
        Ok(chosen) => chosen.clone(),
        Err(poisoned) => poisoned.into_inner().clone(),
    }
}

/// Частота и каналы нынешнего устройства. Замок берётся на одно чтение целиком: частота
/// от нового устройства вместе с числом каналов от прежнего дала бы неверное время.
fn spec_of(shared: &Shared) -> PcmSpec {
    match shared.spec.lock() {
        Ok(spec) => *spec,
        Err(poisoned) => *poisoned.into_inner(),
    }
}

/// Рычаги устройства для того, кто им не владеет.
///
/// Сам `AudioOutput` держит поток cpal и пишущую сторону кольца, а они привязаны к
/// своему потоку и никуда из него не переносятся. Здесь только то, что можно трогать
/// откуда угодно: признаки и счётчики, которые читает колбэк.
#[derive(Clone)]
pub struct OutputControl {
    shared: Arc<Shared>,
}

impl OutputControl {
    /// То же, что `AudioOutput::set_playing`, но из чужого потока: пауза не должна
    /// дожидаться того, кто владеет устройством, — он может стоять на сетевом чтении.
    pub fn set_playing(&self, playing: bool) {
        self.shared.playing.store(playing, Ordering::Release);
    }

    /// Сколько отсчётов устройство забрало из кольца. От этого счётчика считается
    /// позиция в дорожке.
    pub fn delivered_samples(&self) -> u64 {
        self.shared.delivered.load(Ordering::Relaxed)
    }

    /// Частота и каналы того устройства, которое играет сейчас. После смены устройства
    /// здесь уже новые — на них и надо считать время.
    pub fn spec(&self) -> PcmSpec {
        spec_of(&self.shared)
    }

    /// Сколько отсчётов устройство уже проиграло, с досчётом от последнего вызова колбэка.
    ///
    /// Сам счётчик растёт прыжками: он двигается только внутри колбэка. Без досчёта срок полос
    /// спектра наступал бы у целой пачки разом, а на стороне Kotlin поток схлопывающий, и до
    /// экрана из пачки дошла бы только последняя.
    ///
    /// Досчёт идёт от момента, который колбэк отмечает сам, и ни от чего больше, поэтому
    /// сбрасывать его никому не надо. На паузе колбэк зовут так же, он отдаёт тишину и момент
    /// обновляет, — значит, пока устройство живо, досчёт не больше одного его буфера.
    pub fn heard_samples(&self) -> u64 {
        // Момент читается раньше счётчика: колбэк пишет их в обратном порядке.
        let called_at = self.shared.called_at.load(Ordering::Acquire);
        let delivered = self.shared.delivered.load(Ordering::Relaxed);
        if called_at == 0 {
            return delivered;
        }

        let spec = self.spec();
        let per_second = f64::from(spec.sample_rate) * f64::from(spec.channels);
        let since = self
            .shared
            .epoch
            .elapsed()
            .saturating_sub(Duration::from_nanos(called_at));
        delivered + (since.as_secs_f64() * per_second) as u64
    }

    /// Громкость приложения, от 0 до 1. Действует сразу: колбэк читает множитель на каждом
    /// периоде, а не на смене дорожки.
    ///
    /// Слух отзывается на громкость не линейно, поэтому доля возводится в квадрат: на середине
    /// ползунка выходит четверть силы, и это звучит как половина. Кривая живёт здесь, а не у
    /// вызывающего, чтобы у всех она была одна.
    pub fn set_volume(&self, volume: f32) {
        let share = volume.clamp(0.0, 1.0);
        self.shared
            .gain
            .store((share * share).to_bits(), Ordering::Relaxed);
    }

    /// Усиления полос эквалайзера, дБ, от низкой полосы к высокой. Действует сразу, как и
    /// громкость: колбэк сверяет настройки на каждом периоде.
    pub fn set_equalizer(&self, gains_db: [f32; EQ_BANDS]) {
        self.shared.eq.set(gains_db);
    }
}

impl AudioOutput {
    /// Открывает устройство вывода, принятое в системе за основное. Поток создаётся,
    /// но не запускается: устройство трогается с места, когда в кольце накопится буфер.
    ///
    /// `buffer_frames` — желаемая длина буфера устройства в кадрах. `None` отдаёт выбор
    /// устройству; это режим для разбирательств, потому что длина тогда меняется от
    /// запуска к запуску.
    pub fn open(buffer_frames: Option<u32>) -> Result<Self> {
        Self::build(None, buffer_frames, None)
    }

    /// Открывает заново — на том устройстве, которое система считает основным сейчас.
    ///
    /// Зовётся, когда прежнее пропало ([`failed`](Self::failed)): отключили наушники,
    /// выдернули гарнитуру. Общее состояние остаётся тем же, поэтому рычаги, отданные
    /// управляющему потоку при создании плеера, продолжают работать, а счёт отданных
    /// отсчётов не прерывается.
    ///
    /// Выбор устройства при этом сбрасывается: пропало именно выбранное, открывать его заново
    /// незачем — открываем основное. Иначе переоткрытие кончалось бы тем же отказом, и
    /// отключённые наушники снова убивали бы воспроизведение.
    ///
    /// Новый поток строится раньше, чем закроется прежний: если построить не удалось,
    /// прежний остаётся на месте — мёртвый, но описанный, и о неудаче есть кому сказать.
    ///
    /// Кольцо создаётся новое, поэтому всё, что в нём лежало и не прозвучало, пропадает:
    /// в него писали кадрами прежнего устройства, а у нового может быть другая частота.
    pub fn reopen(&mut self, buffer_frames: Option<u32>) -> Result<()> {
        // Признак снимается до постройки, а не после: колбэк ошибок у прежнего потока
        // ещё жив и может сказать о пропаже второй раз. Лишнее переоткрытие безвредно,
        // а стёртая жалоба нового потока означала бы, что мы её больше не услышим.
        self.shared.failed.store(false, Ordering::Release);
        let rebuilt = Self::build(Some(self.shared.clone()), buffer_frames, None)?;
        *self = rebuilt;
        self.forget_callbacks();
        Ok(())
    }

    /// Переходит на другое устройство. `None` — то, которое система считает основным.
    ///
    /// Отличие от [`reopen`](Self::reopen) одно: прежнее устройство живо, и если новое открыть не
    /// удалось, всё остаётся как было — новый поток строится раньше замены. Значит неудачный
    /// выбор в настройках не обрывает музыку.
    pub fn switch_to(&mut self, device: Option<String>, buffer_frames: Option<u32>) -> Result<()> {
        let rebuilt = Self::build(Some(self.shared.clone()), buffer_frames, device)?;
        *self = rebuilt;
        self.forget_callbacks();
        Ok(())
    }

    /// Забывает, когда звали колбэк: прежний поток закрыт, а у нового колбэка ещё не было, и
    /// досчитывать до его первого вызова не от чего. Стоит после замены, а не в постройке:
    /// при смене руками прежний поток живёт, пока новый строится, и его колбэк успел бы
    /// отметиться снова.
    fn forget_callbacks(&self) {
        self.shared.called_at.store(0, Ordering::Release);
    }

    /// Какое устройство выбрано. `None` — основное в системе.
    pub fn chosen_device(&self) -> Option<String> {
        chosen_of(&self.shared)
    }

    /// Пропало ли устройство. Единственный способ узнать: колбэк ошибок cpal, куда oboe
    /// приносит `Disconnected`.
    pub fn failed(&self) -> bool {
        self.shared.failed.load(Ordering::Acquire)
    }

    /// `shared` — общее состояние: `None` при первом открытии, `Some` при переоткрытии.
    /// `device` — имя выбранного устройства или `None` для основного в системе.
    fn build(
        shared: Option<Arc<Shared>>,
        buffer_frames: Option<u32>,
        device: Option<String>,
    ) -> Result<Self> {
        let host = cpal::default_host();
        // Имя сохраняется отдельно от самого устройства: в общее состояние кладётся именно
        // запрошенное, чтобы переоткрытие открывало то же самое. Как называется открытое,
        // видно снаружи через `device_name`.
        let chosen = device;
        let device = match &chosen {
            None => {
                debug!("Сообщение от build из audio_output.rs: устройство не указано, пробуем default_output_device()");
                host.default_output_device()
                    .filter(|dev| dev.default_output_config().is_ok())
                    .or_else(|| {
                    debug!("Сообщение от build из audio_output.rs: default недоступен, перебираем все output_devices()");
                    host.output_devices().ok()?.find_map(|dev| {
                        let name = dev.name().unwrap_or_else(|_| "неизвестно".into());
                        match dev.default_output_config() {
                            Ok(_) => {
                                debug!("Сообщение от build из audio_output.rs: найдено рабочее устройство: {name}");
                                Some(dev)
                            }
                            Err(e) => {
                                debug!("Сообщение от build из audio_output.rs: пропускаем «{name}» ({e:?})");
                                None
                            }
                        }
                    })
                })
                .ok_or_else(|| anyhow!("в системе нет доступных устройств вывода"))?
        }
        Some(target_name) => {
            debug!("Сообщение от build из audio_output.rs: ищем явное устройство по имени: «{target_name}»");
               find_output_device(&host, target_name)
                    .inspect(|dev| {
                       let found_name = dev.name().unwrap_or_else(|_| "неизвестно".into());
                       debug!("Сообщение от build из audio_output.rs: найдено устройство: {found_name}");
                    })
                  .ok_or_else(|| anyhow!("устройства вывода «{target_name}» в системе нет"))?
            }
        };

        let supported = device
        .default_output_config()
        .inspect(|cfg| {
            debug!(
                "Сообщение от build из audio_output.rs: устройство объявило формат: {:?}, {} Гц, {} каналов",
                cfg.sample_format(),
                cfg.sample_rate().0,
                cfg.channels()
            );
        })
        .inspect_err(|err| {
            let dev_name = device.name().unwrap_or_else(|_| "неизвестно".into());
            debug!("Сообщение от build из audio_output.rs: ошибка получения конфига для «{dev_name}»: {err:?}");
        })
        .context("устройство не сообщило свой формат")?;

        if supported.sample_format() != SampleFormat::F32 {
            return Err(anyhow!(
                "устройство принимает отсчёты в формате {:?}, а вывод написан только под f32",
                supported.sample_format()
            ));
        }

        // Число каналов спрашивается здесь, а не после `supported.into()`, потому что
        // на нём считается длина кадра, и ноль каналов означал бы деление на ноль.
        if supported.channels() == 0 {
            return Err(anyhow!("устройство объявило ноль каналов"));
        }
        let bytes_per_frame =
            u32::from(supported.channels()) * supported.sample_format().sample_size() as u32;

        // Устройство может не уметь буфер желаемой длины; тогда берётся ближайшая
        // длина из того, что оно объявило, и она видна снаружи через `buffer_frames`.
        // Объявленный минимум приводится к кадрам: см. `declared_min_frames`.
        let wanted = buffer_frames.map(|frames| match supported.buffer_size() {
            SupportedBufferSize::Range { min, max } => {
                frames.clamp(declared_min_frames(*min, bytes_per_frame), *max)
            }
            SupportedBufferSize::Unknown => frames,
        });
        // Диапазон запоминается строкой до `supported.into()`, которое забирает
        // `supported` себе: в журнале нужно то, что объявило устройство, а не то, что из
        // этого вышло у нас. Минимум печатается и как объявлен, и как понят, иначе по
        // журналу не отличить исправленный случай от прежнего.
        let declared_buffer = match supported.buffer_size() {
            SupportedBufferSize::Range { min, max } => format!(
                "{min}..{max}, минимум понят как {} кадров",
                declared_min_frames(*min, bytes_per_frame)
            ),
            SupportedBufferSize::Unknown => "не объявлен".to_string(),
        };
        let device_name = device.name().unwrap_or_else(|_| "имя неизвестно".to_string());

        let mut config: StreamConfig = supported.into();
        config.buffer_size = match wanted {
            Some(frames) => BufferSize::Fixed(frames),
            None => BufferSize::Default,
        };
        // Когда длину выбирает устройство, заранее она неизвестна, и на ожидание берётся
        // та, которую мы всё равно считаем разумной.
        let buffer_frames = wanted.unwrap_or(DEVICE_BUFFER_FRAMES);

        let spec = PcmSpec {
            sample_rate: config.sample_rate.0,
            channels: config.channels,
        };
        let ring_samples = buffer_frames as usize * usize::from(spec.channels) * RING_BUFFERS;
        let (producer, mut consumer) = RingBuffer::<f32>::new(ring_samples);

        // Всё согласовано, и дальше эти числа не меняются до закрытия плеера. Строка
        // стоит до `build_output_stream`, чтобы числа были в журнале и тогда, когда поток
        // не построится. Длина буфера — большее из желаемой и объявленного устройством
        // минимума, и из неё считается всё остальное: кольцо (`RING_BUFFERS` раз), стартовая
        // тишина (`prefill`, ровно один буфер) и срок ожидания места (`period`, тоже ровно
        // один буфер). Отдельных строк для `prefill` и `period` нет именно поэтому: они
        // равны длительности буфера. Задержка паузы — тоже она: колбэк начинает отдавать
        // тишину, а то, что уже отдано драйверу, доиграет.
        //
        // Просьба и выданное — разные вещи: `set_buffer_capacity_in_frames` в oboe только
        // просьба, а сколько дали, знает `get_buffer_capacity_in_frames`, которого cpal
        // наружу не отдаёт. В журнале поэтому запрошенное.
        info!(
            "вывод открыт: {device_name}, {} Гц, {} кан.; буфер устройства объявлен \
             {declared_buffer}, взят {buffer_frames} кадров = {:.0} мс; кольцо \
             {ring_samples} отсчётов = {:.0} мс",
            spec.sample_rate,
            spec.channels,
            1000.0 * f64::from(buffer_frames) / f64::from(spec.sample_rate),
            1000.0 * ring_samples as f64 / f64::from(spec.channels) / f64::from(spec.sample_rate),
        );

        let shared = match shared {
            // Переоткрытие: частота и каналы могли смениться вместе с устройством, а
            // просьба выбросить кольцо относилась к прежнему, которого больше нет.
            Some(shared) => {
                match shared.spec.lock() {
                    Ok(mut held) => *held = spec,
                    Err(poisoned) => *poisoned.into_inner() = spec,
                }
                match shared.chosen.lock() {
                    Ok(mut held) => *held = chosen,
                    Err(poisoned) => *poisoned.into_inner() = chosen,
                }
                shared.discard.store(false, Ordering::Release);
                shared
            }
            None => Arc::new(Shared::new(spec, chosen)),
        };
        let callback_shared = shared.clone();
        let error_shared = shared.clone();
        // Частота у потока своя и до его закрытия не меняется, поэтому эквалайзер
        // создаётся вместе с потоком, и его коэффициенты всегда посчитаны под неё.
        let mut equalizer = Equalizer::new(spec.sample_rate, spec.channels);

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &OutputCallbackInfo| {
                    callback_shared.callbacks.fetch_add(1, Ordering::Relaxed);

                    // Просьба выбросить очередь. Чтение и списание целой порции — это
                    // два атомарных счётчика, память здесь не трогается.
                    if callback_shared.discard.swap(false, Ordering::Acquire) {
                        equalizer.reset();
                        let waiting = consumer.slots();
                        if waiting > 0 {
                            if let Ok(chunk) = consumer.read_chunk(waiting) {
                                chunk.commit_all();
                            }
                        }
                    }

                    // Пауза. Кольцо не трогаем: то, что в нём лежит, дождётся
                    // продолжения и заиграет с того же места.
                    if !callback_shared.playing.load(Ordering::Acquire) {
                        data.fill(0.0);
                        callback_shared.mark_called();
                        return;
                    }

                    let (popped, rest) = consumer.pop_partial_slice(data);
                    callback_shared
                        .delivered
                        .fetch_add(popped.len() as u64, Ordering::Relaxed);
                    callback_shared.mark_called();

                    equalizer.sync(&callback_shared.eq);
                    equalizer.process(popped);

                    // Громкость — умножение на месте, здесь и только здесь. В потоке
                    // декодирования это было бы дешевле, но менялась бы громкость не сразу, а
                    // когда устройство доиграет кольцо, — до 372 мс на ползунке.
                    let gain = f32::from_bits(callback_shared.gain.load(Ordering::Relaxed));
                    for sample in popped.iter_mut() {
                        *sample *= gain;
                    }
                    if !rest.is_empty() {
                        // Кадры кончились. Отдать что-то колбэк обязан, и единственное,
                        // что можно отдать, — тишина; счётчик говорит, сколько её было.
                        callback_shared
                            .starved
                            .fetch_add(rest.len() as u64, Ordering::Relaxed);
                        rest.fill(0.0);
                    }
                },
                move |err| {
                    // Пропажу устройства cpal приносит сюда же, обычной ошибкой потока.
                    // Отличать её от прочих незачем: любая ошибка потока означает, что
                    // кадры больше никто не забирает, и лечится она одинаково.
                    error_shared.failed.store(true, Ordering::Release);
                    match error_shared.errors.lock() {
                        Ok(mut errors) => errors.push(err.to_string()),
                        Err(poisoned) => poisoned.into_inner().push(err.to_string()),
                    }
                },
                None,
            )
            .context("поток вывода не построился")?;

        let period =
            Duration::from_secs_f64(f64::from(buffer_frames) / f64::from(spec.sample_rate));

        Ok(AudioOutput {
            stream,
            producer,
            started: false,
            prefill: buffer_frames as usize * usize::from(spec.channels),
            buffer_frames,
            device_name,
            period,
            shared,
        })
    }

    /// Что принимает устройство: частота и число каналов. Под это настраивается
    /// `FrameConverter`.
    pub fn spec(&self) -> PcmSpec {
        spec_of(&self.shared)
    }

    /// Рычаги, которые можно отдать в другой поток. Переоткрытие устройства их не
    /// обесценивает: за ними общее состояние, а не поток.
    pub fn control(&self) -> OutputControl {
        OutputControl {
            shared: self.shared.clone(),
        }
    }

    /// Длина буфера устройства в кадрах — та, на которой оно в итоге сошлось.
    pub fn buffer_frames(&self) -> u32 {
        self.buffer_frames
    }

    /// Отдаёт устройству столько отсчётов, сколько помещается в кольцо, и возвращает
    /// это число. Не ждёт: остаток — забота вызывающего.
    pub fn try_write(&mut self, samples: &[f32]) -> Result<usize> {
        let (pushed, _) = self.producer.push_partial_slice(samples);

        // Устройство трогается с места, только когда ему есть что играть. Запусти его
        // раньше — и первые вызовы колбэка придут на пустое кольцо, то есть дорожка
        // начнётся с тишины.
        if self.queued_samples() >= self.prefill {
            self.start()?;
        }
        Ok(pushed.len())
    }

    /// Играть или молчать. В тишине колбэк не трогает кольцо, так что после
    /// продолжения звук пойдёт ровно с того места, где оборвался.
    ///
    /// Сам поток cpal при этом продолжает идти. Остановить его значило бы полагаться на
    /// `Stream::pause`, которого ALSA на многих устройствах не умеет, а ошибку оттуда
    /// cpal проглатывает — то есть пауза работала бы через раз и молча.
    pub fn set_playing(&self, playing: bool) {
        self.shared.playing.store(playing, Ordering::Release);
    }

    /// Просит выбросить всё, что лежит в кольце. Сработает на ближайшем вызове колбэка,
    /// то есть звук оборвётся в пределах одного периода.
    ///
    /// Заодно запускает устройство, если оно ещё стоит: выбросить кольцо может только
    /// колбэк, а пока поток не идёт, колбэка не будет — и просьба дождалась бы кадров
    /// уже следующей дорожки, выбросив заодно и их.
    pub fn discard_queued(&mut self) -> Result<()> {
        self.shared.discard.store(true, Ordering::Release);
        self.start()
    }

    /// Сколько ещё будет звучать то, что уже отдано: кольцо плюс буфер самого
    /// устройства. Ровно на столько звук отстаёт от кадра, который сейчас кладут.
    pub fn queued_ahead(&self) -> Duration {
        let spec = self.spec();
        let samples = self.queued_samples() as f64;
        let per_second = f64::from(spec.sample_rate) * f64::from(spec.channels);
        Duration::from_secs_f64(samples / per_second) + self.period
    }

    /// Сколько длится один буфер устройства. По этому сроку меряется ожидание места.
    pub fn period(&self) -> Duration {
        self.period
    }

    /// Отдаёт отсчёты устройству, дожидаясь места в кольце. Вызывать из колбэка или из
    /// потока, который держит что-то нужное другим, нельзя: вызов блокирует.
    pub fn write(&mut self, samples: &[f32]) -> Result<()> {
        let mut rest = samples;
        while !rest.is_empty() {
            let written = self.try_write(rest)?;
            rest = &rest[written..];
            if rest.is_empty() {
                break;
            }
            // Кольцо полно. Ждать в нём места, не запустив устройство, значит ждать
            // вечно: забирать кадры некому.
            self.start()?;
            if self.producer.is_abandoned() {
                return Err(anyhow!("поток вывода закрылся, отдавать кадры некому"));
            }
            // Место освободится не раньше, чем колбэк заберёт очередной буфер.
            sleep(self.period / 2);
        }
        Ok(())
    }

    /// Имя устройства, на которое попал вывод.
    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    /// Сколько раз устройство позвало колбэк. Ноль во время игры означает, что поток
    /// стоит, и разбираться надо с cpal, а не с кадрами.
    pub fn callbacks(&self) -> u64 {
        self.shared.callbacks.load(Ordering::Relaxed)
    }

    /// Сколько отсчётов устройство забрало из кольца.
    pub fn delivered_samples(&self) -> u64 {
        self.shared.delivered.load(Ordering::Relaxed)
    }

    /// Запущен ли поток.
    pub fn started(&self) -> bool {
        self.started
    }

    /// Сколько отсчётов лежит в кольце и ждёт устройства.
    pub fn queued_samples(&self) -> usize {
        self.producer.buffer().capacity() - self.producer.slots()
    }

    fn start(&mut self) -> Result<()> {
        if self.started {
            return Ok(());
        }
        self.stream.play().context("поток вывода не запустился")?;
        self.started = true;
        Ok(())
    }

    /// Сколько отсчётов устройство получило тишиной, потому что кадры не поспели.
    /// Ноль за всю дорожку — признак того, что тракт успевает.
    pub fn starved_samples(&self) -> u64 {
        self.shared.starved.load(Ordering::Relaxed)
    }

    /// Ошибки, о которых сообщал поток за время работы.
    pub fn errors(&self) -> Vec<String> {
        match self.shared.errors.lock() {
            Ok(errors) => errors.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    /// Доигрывает всё, что уже отдано, и только потом закрывает поток. Закрывать поток
    /// через `drop` посреди звука нельзя: устройство обрывает его на полуслове, и это
    /// слышно щелчком на каждой смене дорожки.
    pub fn drain_and_stop(mut self) -> Result<()> {
        // Дорожка могла оказаться короче одного буфера — тогда устройство ещё стоит,
        // и ожидание пустого кольца никогда бы не кончилось.
        self.start()?;
        while self.queued_samples() > 0 {
            if self.producer.is_abandoned() {
                break;
            }
            sleep(self.period / 2);
        }
        // Кольцо пусто, но то, что из него забрали последним, устройство ещё играет.
        sleep(self.period);
        // И только теперь поток закрывается — уже на тишине.
        drop(self.stream);
        Ok(())
    }
}
