// Внутренности потока декодирования: дорожка в работе и подача кадров устройству.

use std::path::Path;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use super::{Order, Progress, Report};
use crate::network::QuicSession;
use crate::player::converter::FrameConverter;
use crate::player::decoder::TrackDecoder;
use crate::player::fft_processor::FFT_SIZE;
use crate::player::file_scanner::TrackData;
use crate::player::output::{AudioOutput, OutputControl, DEVICE_BUFFER_FRAMES};
use crate::player::player_core::{BandsListener, Incoming, PlayerError};
use crate::player::spectrum::SpectrumThread;
use crate::player::quic_source::{QuicSource, TrackCancel};

/// Дорожка в работе: декодер, конвертер и то, что уже сконвертировано, но ещё не влезло
/// в кольцо вывода.
struct Track {
    decoder: TrackDecoder,
    converter: FrameConverter,
    /// Пакет от декодера.
    packet: Vec<f32>,
    /// Кадры, готовые для устройства.
    ready: Vec<f32>,
    /// Сколько из `ready` уже отдано.
    ready_from: usize,
    /// Декодер дошёл до конца дорожки и отдал хвост ресемплера.
    decoded_to_end: bool,
    duration: Option<Duration>,
}

/// Чем закончился один проход подачи кадров.
enum Pumped {
    /// Есть чем заняться прямо сейчас.
    Busy,
    /// Кольцо полно — надо дать устройству забрать.
    Wait,
    /// Дорожка кончилась, и последний её кадр уже прозвучал.
    Finished,
}

pub(super) struct Decode {
    output: AudioOutput,
    reports: Sender<Incoming>,
    progress: Arc<Mutex<Progress>>,
    /// Подключение к серверу для сетевых дорожек.
    session: Option<Arc<QuicSession>>,
    /// Кому отдавать окна отсчётов на спектр. Считает их отдельный поток: преобразование Фурье и
    /// вызов через границу в Kotlin к подаче кадров отношения не имеют и только задерживали её.
    spectrum: SpectrumThread,
    track: Option<Track>,
    /// Поколение дорожки, которая сейчас в работе. Ноль — дорожки нет.
    generation: u64,
    /// Стоим на паузе: кольцо выброшено, и наполнять его заново незачем — на продолжении
    /// оно наполнится с нуля, как на старте дорожки. Пока признак стоит, поток спит на
    /// приказе. Снимается продолжением и открытием новой дорожки; других способов снова
    /// заиграть нет.
    paused: bool,
    /// Сколько ждать, когда кольцо полно: половина буфера устройства.
    wait: Duration,
    /// Показание счётчика голодания на последней строке журнала о нём, и когда она была.
    starved_said: u64,
    starved_at: Instant,
}

impl Decode {
    pub(super) fn run(
        inbox: Receiver<Order>,
        reports: Sender<Incoming>,
        progress: Arc<Mutex<Progress>>,
        session: Option<Arc<QuicSession>>,
        bands: Box<dyn BandsListener>,
        ready: Sender<Result<OutputControl, PlayerError>>,
    ) {
        let output = match AudioOutput::open(Some(DEVICE_BUFFER_FRAMES)) {
            Ok(output) => output,
            Err(err) => {
                // Если хозяин уже ушёл, о неудаче рассказывать некому.
                drop(ready.send(Err(PlayerError::device(err))));
                return;
            }
        };
        // Пока дорожки нет, устройство должно молчать: кольцо пусто, и колбэк иначе
        // считал бы каждую тишину за опоздание кадров.
        output.set_playing(false);
        let wait = output.period() / 2;
        if ready.send(Ok(output.control())).is_err() {
            return;
        }
        // Рычаги устройства достаются и потоку спектра: он сам решает, когда полосы зазвучали, по
        // счётчику отданных отсчётов.
        let spectrum = SpectrumThread::start(bands, output.control());
        let decode = Decode {
            output,
            reports,
            progress,
            session,
            spectrum,
            track: None,
            generation: 0,
            paused: false,
            wait,
            starved_said: 0,
            starved_at: Instant::now(),
        };
        decode.serve(inbox);
    }

    fn serve(mut self, inbox: Receiver<Order>) {
        loop {
            // Устройство могло пропасть, пока мы спали: наушники отключают когда хотят,
            // и узнаём мы об этом не приказом, а признаком у устройства.
            if self.output.failed() {
                self.switch_output();
            }

            // Приказы вперёд кадров: брошенную дорожку надо бросить сразу, а не когда
            // наполнится кольцо.
            loop {
                match inbox.try_recv() {
                    Ok(order) => {
                        if !self.apply(order) {
                            return;
                        }
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return,
                }
            }

            // Ни дорожки, ни паузы — ждать приказ, не занимая процессор. На паузе не
            // просто спим: кольцо на ней выброшено (см. `drop_queued`), и наполнять его
            // заново значило бы копить то же самое, что только что выбросили.
            if self.track.is_none() || self.paused {
                match inbox.recv() {
                    Ok(order) => {
                        if !self.apply(order) {
                            return;
                        }
                    }
                    Err(_) => return,
                }
                continue;
            }

            match self.pump() {
                Ok(Pumped::Busy) => {}
                Ok(Pumped::Wait) => {
                    // Приказ прервёт ожидание раньше срока, поэтому смена дорожки
                    // остаётся мгновенной даже с полным кольцом.
                    match inbox.recv_timeout(self.wait) {
                        Ok(order) => {
                            if !self.apply(order) {
                                return;
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {}
                        Err(RecvTimeoutError::Disconnected) => return,
                    }
                }
                Ok(Pumped::Finished) => {
                    let generation = self.generation;
                    self.forget();
                    self.report(Report::Finished(generation));
                }
                Err(err) => self.fail(err),
            }

            self.report_starvation();
        }
    }

    /// Голодание: сколько отсчётов устройство получило тишиной, не дождавшись кадров. Это
    /// и есть рывки на слух, и другого способа отличить их от чего угодно ещё нет.
    ///
    /// Строка пишется, только пока счётчик растёт, и не чаще раза в секунду: проход этого
    /// цикла идёт десятки раз в секунду, а запись в журнал на телефоне стоит дороже, чем
    /// само голодание, — печать на каждом проходе рвала бы звук сама.
    fn report_starvation(&mut self) {
        let starved = self.output.starved_samples();
        if starved == self.starved_said {
            return;
        }
        let now = Instant::now();
        if now.duration_since(self.starved_at) < Duration::from_secs(1) {
            return;
        }

        let spec = self.output.spec();
        let frames = (starved - self.starved_said) / u64::from(spec.channels);
        let silence = 1000.0 * frames as f64 / f64::from(spec.sample_rate);

        self.starved_said = starved;
        self.starved_at = now;
    }

    /// Выполняет приказ. `false` означает «выключаемся».
    fn apply(&mut self, order: Order) -> bool {
        match order {
            Order::Open {
                track,
                cancel,
                generation,
            } => self.open(track, cancel, generation),
            Order::Seek {
                position,
                generation,
            } => self.seek(position, generation),
            Order::Pause(paused) => {
                self.paused = paused;
                if paused {
                    self.drop_queued();
                }
            }
            Order::Drop => self.forget(),
            Order::SetOutputDevice(device) => self.change_output(device),
            Order::SetSession(session) => self.session = session,
            Order::Shutdown => return false,
        }
        true
    }

    /// Открывает дорожку и оставляет её играть. Ожидание здесь может быть долгим —
    /// столько, сколько сервер не отвечает; обрывается оно отменой снаружи, и тогда
    /// открытие кончается ошибкой, о которой спрашивавший уже не спросит.
    fn open(&mut self, track: TrackData, cancel: TrackCancel, generation: u64) {
        self.forget();
        self.generation = generation;
        // Новая дорожка идёт играть: паузы на ней ещё не было.
        self.paused = false;

        match self.open_track(&track, &cancel) {
            Ok(opened) => {
                self.publish(Progress {
                    generation,
                    offset: Duration::ZERO,
                    delivered_at_start: self.output.delivered_samples(),
                    duration: opened.duration,
                });
                self.track = Some(opened);
                self.report(Report::Started(generation));
            }
            Err(err) => {
                self.generation = 0;
                self.report(Report::Failed(generation, PlayerError::track(err)));
            }
        }
    }

    fn open_track(&self, track: &TrackData, cancel: &TrackCancel) -> Result<Track> {
        let decoder = if track.is_local {
            TrackDecoder::open_file(Path::new(&track.track_url))?
        } else {
            // Сетевая дорожка читается прямо из QUIC: локальный прокси для
            // воспроизведения больше не нужен.
            let session = self.session.as_ref().ok_or_else(|| {
                anyhow!(
                    "дорожка «{}» лежит на сервере, а подключения нет",
                    track.track_name
                )
            })?;
            let source = QuicSource::open(session, &track.track_url, cancel)?;
            TrackDecoder::open(Box::new(source), extension_of(&track.track_url))?
        };

        let converter = FrameConverter::new(decoder.spec(), self.output.spec())?;
        Ok(Track {
            duration: decoder.duration(),
            decoder,
            converter,
            packet: Vec::new(),
            ready: Vec::new(),
            ready_from: 0,
            decoded_to_end: false,
        })
    }

    /// Перематывает текущую дорожку. Приказ о дорожке, которой уже нет, пропускается:
    /// пока он шёл, её могли сменить.
    fn seek(&mut self, position: Duration, generation: u64) {
        if self.track.is_none() || generation != self.generation {
            return;
        }

        // То, что уже уехало в устройство, относится к прежнему месту дорожки.
        self.discard();
        // И полосы, посчитанные из выброшенного, — тоже: без этого визуализатор ещё целое
        // кольцо показывал бы то, что играло до перемотки.
        self.spectrum.clear();
        let delivered = self.output.delivered_samples();

        let sought = match self.track.as_mut() {
            Some(track) => track.decoder.seek(position),
            None => return,
        };
        match sought {
            Ok(actual) => {
                let duration = match self.track.as_mut() {
                    Some(track) => {
                        track.converter.reset();
                        track.ready.clear();
                        track.ready_from = 0;
                        track.decoded_to_end = false;
                        track.duration
                    }
                    None => return,
                };
                self.publish(Progress {
                    generation,
                    offset: actual,
                    delivered_at_start: delivered,
                    duration,
                });
                // Место дорожки теперь другое, и опрос об этом узнал бы только на
                // следующем тике — а до него ползунок успел бы сесть на прежнее место.
                self.report(Report::PositionChanged(generation));
            }
            Err(err) => self.fail(err),
        }
    }

    /// Один проход: донести остаток, а если он кончился — декодировать следующий пакет.
    fn pump(&mut self) -> Result<Pumped> {
        let Some(track) = self.track.as_mut() else {
            return Err(anyhow!("дорожки нет, а кадры просят"));
        };

        if track.ready_from < track.ready.len() {
            let from = track.ready_from;
            let written = self.output.try_write(&track.ready[from..])?;
            track.ready_from += written;
            if written > 0 {
                // Здесь только копия окна и счёт того, когда оно зазвучит: и то и другое дешёво.
                // Преобразование Фурье и отдача наружу — дело потока спектра.
                let window: Vec<f32> = track.ready[from..from + written]
                    .iter()
                    .take(FFT_SIZE)
                    .copied()
                    .collect();

                // Отсчёт, на котором эти полосы зазвучат: всё, что лежало в кольце перед
                // нашей порцией, плюс буфер самого устройства — то, что оно уже забрало,
                // но ещё не проиграло. `delivered + queued` меняется только на записи:
                // колбэк одно уменьшает и другое увеличивает, — поэтому сумма берётся
                // целиком, а вычитание с защитой: колбэк мог успеть забрать часть нашей
                // порции между записью и этим чтением.
                let spec = self.output.spec();
                let buffered =
                    u64::from(self.output.buffer_frames()) * u64::from(spec.channels);
                let total = self.output.delivered_samples() + self.output.queued_samples() as u64;
                let at = total.saturating_sub(written as u64) + buffered;
                self.spectrum.feed(window, at);
            }
            return Ok(if written > 0 { Pumped::Busy } else { Pumped::Wait });
        }

        if !track.decoded_to_end {
            track.ready.clear();
            track.ready_from = 0;
            if track.decoder.next_frames(&mut track.packet)? {
                track.converter.push(&track.packet, &mut track.ready)?;
            } else {
                // Дорожка кончилась, но внутри ресемплера остались кадры.
                track.converter.drain(&mut track.ready)?;
                track.decoded_to_end = true;
            }
            // Пусто — не беда: конвертер копит кадры до полной порции.
            return Ok(Pumped::Busy);
        }

        // Отдавать больше нечего; дорожка кончится, когда устройство доиграет кольцо.
        if self.output.queued_samples() == 0 {
            Ok(Pumped::Finished)
        } else {
            Ok(Pumped::Wait)
        }
    }

    /// Бросает дорожку и выбрасывает недоигранное. Без дорожки выбрасывать нечего —
    /// и трогать устройство незачем: просьба очистить кольцо запускает его, а пустое
    /// кольцо запускать не за чем.
    fn forget(&mut self) {
        if self.track.take().is_some() {
            self.discard();
        }
        self.spectrum.clear();
        self.generation = 0;
        self.publish(Progress::none());
    }

    /// Выбрасывает на паузе всё накопленное: и кольцо, и очередь полос.
    ///
    /// Продолжение после этого ведёт себя как старт дорожки: кольцо и очередь наполняются в
    /// такт, и картинка идёт с первого кадра — держать очередь и отдавать её по сроку
    /// незачем, потому что звука, к которому она относилась, больше нет.
    ///
    /// Цена: тот звук декодер уже отдал, и песня продолжится на длину кольца дальше — до
    /// 372 мс на динамике и до 1224 мс на наушниках. Позиция об этом знает: место отсчёта
    /// сдвигается на выброшенное, иначе показанная позиция отставала бы от музыки, и на
    /// каждой паузе всё сильнее.
    ///
    /// Без дорожки выбрасывать нечего, и трогать устройство незачем: просьба очистить
    /// кольцо запускает его, а пустое кольцо запускать не за чем.
    fn drop_queued(&mut self) {
        if self.track.is_none() {
            return;
        }

        // Звук уже заглушён управляющим потоком — он делает это раньше, чем отдаёт приказ о
        // паузе, — поэтому колбэк из кольца больше не берёт, и оба числа не убегут.
        let spec = self.output.spec();
        let dropped = self.output.queued_samples() as u64;
        let delivered = self.output.delivered_samples();
        let progress = self.current_progress();
        let heard = progress.position(delivered, spec);

        self.discard();
        self.spectrum.clear();

        let frames = dropped / u64::from(spec.channels);
        let skipped = Duration::from_secs_f64(frames as f64 / f64::from(spec.sample_rate));
        self.publish(Progress {
            generation: progress.generation,
            offset: heard + skipped,
            delivered_at_start: delivered,
            duration: progress.duration,
        });
        // На паузе опрос позиции останавливается, поэтому новое место надо назвать самим:
        // иначе на экране осталось бы то, что играло до выброса кольца.
        self.report(Report::PositionChanged(progress.generation));
    }

    /// Устройство пропало — отключили наушники, выдернули гарнитуру. Переоткрываем на
    /// том, которое система считает основным теперь, и оставляем дорожку стоять: снятые
    /// наушники не должны означать музыку из динамика на весь автобус. Играть дальше —
    /// обычным продолжением, то есть нажатием.
    fn switch_output(&mut self) {
        let lost = self.output.device_name().to_string();

        // Позиция считается до переоткрытия и по прежнему устройству: прозвучало ровно
        // то, что оно успело забрать, а лежавшее в кольце пропадёт вместе с кольцом.
        let heard = self.heard_position();

        if let Err(err) = self.output.reopen(Some(DEVICE_BUFFER_FRAMES)) {
            // Устройства нет вовсе — играть больше нечем, и это уже беда дорожки.
            let generation = self.generation;
            self.forget();
            self.report(Report::Failed(generation, PlayerError::device(err)));
            return;
        }

        // Молчание ставится здесь, а не в управляющем потоке: тот узнает о смене
        // сообщением, а до тех пор новое устройство уже играло бы пустое кольцо.
        self.output.set_playing(false);
        self.paused = true;

        if self.settle_output(heard) {
            self.report(Report::OutputChanged(self.generation));
        }
    }

    /// Выбрали другое устройство руками. Дорожка продолжает играть: смену сделали нарочно, и
    /// обрывать её незачем.
    ///
    /// Если новое устройство не открылось — всё остаётся на прежнем, живом: `switch_to` строит
    /// поток раньше, чем заменяет. Наружу уходит жалоба, а не обрыв дорожки.
    fn change_output(&mut self, device: Option<String>) {
        // Просят то же, что открыто, — делать нечего. Это не мелочь: `switch_to` строит новый
        // поток раньше, чем закрывает прежний, и на сыром устройстве ALSA, которое монопольно,
        // второй поток к тому же устройству просто не открылся бы. Сюда приходит и первая
        // настройка при запуске, когда открыто уже то, что просят.
        if device == self.output.chosen_device() {
            return;
        }

        let heard = self.heard_position();

        if let Err(err) = self.output.switch_to(device, Some(DEVICE_BUFFER_FRAMES)) {
            self.report(Report::DeviceFailed(PlayerError::device(err)));
            return;
        }

        if self.settle_output(heard) {
            self.report(Report::PositionChanged(self.generation));
        }
    }

    /// Место дорожки, до которого устройство успело доиграть. Считается по нынешнему устройству,
    /// поэтому спрашивать надо до переоткрытия.
    fn heard_position(&self) -> Duration {
        self.current_progress()
            .position(self.output.delivered_samples(), self.output.spec())
    }

    /// Приводит в порядок всё, что зависит от устройства, после его смены: срок ожидания места,
    /// очередь полос, готовые кадры, конвертер и место в дорожке.
    ///
    /// `false` означает, что дорожки нет или она сорвалась, — рассказывать о смене тогда нечего.
    fn settle_output(&mut self, heard: Duration) -> bool {
        // Ожидание места меряется буфером устройства, а буфер теперь другой.
        self.wait = self.output.period() / 2;
        // Полосы посчитаны из кадров, пропавших вместе с прежним кольцом.
        self.spectrum.clear();

        let spec = self.output.spec();
        let rebuilt = match self.track.as_mut() {
            Some(track) => {
                // Готовые кадры — в формате прежнего устройства, и конвертер настроен на
                // него же: у нового частота может быть другой.
                track.ready.clear();
                track.ready_from = 0;
                FrameConverter::new(track.decoder.spec(), spec).map(|converter| {
                    track.converter = converter;
                    track.duration
                })
            }
            None => return false,
        };

        let duration = match rebuilt {
            Ok(duration) => duration,
            Err(err) => {
                self.fail(err);
                return false;
            }
        };

        self.publish(Progress {
            generation: self.generation,
            offset: heard,
            delivered_at_start: self.output.delivered_samples(),
            duration,
        });
        true
    }

    fn fail(&mut self, err: anyhow::Error) {
        let generation = self.generation;
        self.forget();
        self.report(Report::Failed(generation, PlayerError::track(err)));
    }

    /// Выбрасывает недоигранное. О неудаче рассказывает сообщением: вызывают это и из
    /// `fail`, и городить там повторный заход было бы кругом.
    fn discard(&mut self) {
        if let Err(err) = self.output.discard_queued() {
            let generation = self.generation;
            self.report(Report::Failed(generation, PlayerError::device(err)));
        }
    }

    fn current_progress(&self) -> Progress {
        *self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn publish(&self, progress: Progress) {
        *self
            .progress
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = progress;
    }

    /// Неудача отправки означает, что управляющий поток кончился; тогда и рассказывать
    /// некому, а уйдём мы по закрытой очереди приказов.
    fn report(&self, report: Report) {
        drop(self.reports.send(Incoming::Report(report)));
    }
}

/// Расширение из адреса дорожки — подсказка разборщику контейнера. Пустая строка, если
/// его в адресе нет: тогда формат определится по содержимому.
fn extension_of(url: &str) -> &str {
    match url.rsplit_once('.') {
        Some((_, extension)) if !extension.is_empty() && extension.len() <= 5 => extension,
        _ => "",
    }
}
