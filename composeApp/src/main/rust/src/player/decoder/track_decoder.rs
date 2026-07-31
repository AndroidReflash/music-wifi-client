use std::fs::File;
use std::path::Path;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use symphonia::core::codecs::audio::{AudioDecoder, AudioDecoderOptions};
use symphonia::core::formats::probe::Hint;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo, TrackType};
use symphonia::core::io::{MediaSource, MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::units::{Time, TimeBase};
use symphonia::default::{get_codecs, get_probe};

const MAX_CONSECUTIVE_ERRORS: u32 = 30;
// Формат кадров, которые отдаёт декодер. Кадр — по одному отсчёту на каждый канал,
/// снятые в один и тот же момент времени.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PcmSpec {
    /// Частота дискретизации дорожки в герцах: сколько кадров приходится на секунду звука.
    pub sample_rate: u32,
    /// Сколько каналов в кадре. `u16`, потому что такой тип у числа каналов в cpal.
    pub channels: u16,
}

/// Декодер одной дорожки: на входе сжатый поток, на выходе кадры `f32`.
///
/// Источник берётся как `Box<dyn MediaSource>` — это `Read + Seek + Send + Sync`.
/// Файл им уже является; сюда же потом встанет источник поверх QUIC, когда у него
/// появится чтение диапазона байтов.
pub struct TrackDecoder {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn AudioDecoder>,
    track_id: u32,
    spec: PcmSpec,
    time_base: Option<TimeBase>,
    duration: Option<Duration>,
    consecutive_errors: u32,
}

impl TrackDecoder {
    pub fn open_file(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("не открыть {}", path.display()))?;
        let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or_default();
        Self::open(Box::new(file), extension)
    }

    /// `extension` — подсказка, каким разборщиком контейнера пробовать первым; пустая строка,
    /// если расширение неизвестно. Содержимое всё равно проверяется по сигнатуре, подсказка
    /// только экономит перебор.
    pub fn open(source: Box<dyn MediaSource>, extension: &str) -> Result<Self> {
        let stream = MediaSourceStream::new(source, MediaSourceStreamOptions::default());

        let mut hint = Hint::new();
        if !extension.is_empty() {
            hint.with_extension(extension);
        }

        let format = get_probe()
            .probe(
                &hint,
                stream,
                FormatOptions::default(),
                MetadataOptions::default(),
            )
            .context("контейнер не распознан")?;

        let track = format
            .default_track(TrackType::Audio)
            .ok_or_else(|| anyhow!("в потоке нет звуковой дорожки"))?;
        let track_id = track.id;

        let params = track
            .codec_params
            .as_ref()
            .and_then(|params| params.audio())
            .ok_or_else(|| anyhow!("кодек дорожки не определён"))?;

        let sample_rate = params
            .sample_rate
            .ok_or_else(|| anyhow!("дорожка не объявляет частоту дискретизации"))?;
        let channel_count = params
            .channels
            .as_ref()
            .ok_or_else(|| anyhow!("дорожка не объявляет каналы"))?
            .count();
        if channel_count == 0 {
            return Err(anyhow!("в дорожке ноль каналов, играть нечего"));
        }
        let channels = u16::try_from(channel_count)
            .map_err(|_| anyhow!("каналов {}, больше, чем бывает", channel_count))?;

        // Длительность контейнер хранит в тактах своей шкалы времени, `time_base` переводит
        // такты в секунды. Нет шкалы или нет длительности — длительность неизвестна, и это
        // обычное дело для потокового источника.
        let time_base = track.time_base;
        let duration = match (time_base, track.duration) {
            (Some(time_base), Some(duration)) => time_base
                .calc_duration(duration)
                .map(|time| Duration::from_secs_f64(time.as_secs_f64())),
            _ => None,
        };

        let decoder = get_codecs()
            .make_audio_decoder(params, &AudioDecoderOptions::default())
            .context("для кодека дорожки нет декодера")?;

        let errors: u32 = 0;
        Ok(TrackDecoder {
            format,
            decoder,
            track_id,
            spec: PcmSpec {
                sample_rate,
                channels,
            },
            time_base,
            duration,
            consecutive_errors: errors,
        })
    }

    pub fn spec(&self) -> PcmSpec {
        self.spec
    }

    pub fn duration(&self) -> Option<Duration> {
        self.duration
    }

    /// Перематывает дорожку и возвращает место, куда попала на самом деле: контейнер
    /// умеет вставать только на границу пакета, и это бывает заметно раньше запрошенного.
    ///
    /// Кадры, уже отданные наружу, к новому месту отношения не имеют — их выбрасывает
    /// вызывающий, вместе с тем, что успело уехать в устройство.
    pub fn seek(&mut self, position: Duration) -> Result<Duration> {
        let time = Time::try_from_secs_f64(position.as_secs_f64())
            .ok_or_else(|| anyhow!("на {:?} перемотать нельзя", position))?;

        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .context("перемотка не удалась")?;

        // После перемотки декодер обязан забыть всё, что помнил о прошлых пакетах:
        // следующий пакет ему разрывный.
        self.decoder.reset();

        let time_base = self
            .time_base
            .ok_or_else(|| anyhow!("у дорожки нет шкалы времени, место после перемотки неизвестно"))?;
        let actual = time_base
            .calc_time(seeked.actual_ts)
            .ok_or_else(|| anyhow!("место после перемотки не переводится в секунды"))?
            .as_secs_f64();

        // Метка времени бывает отрицательной: так контейнер отмечает служебные кадры
        // задержки кодера перед началом дорожки. Для позиции это ноль.
        Ok(Duration::from_secs_f64(actual.max(0.0)))
    }

    /// Декодирует следующий пакет в `out`: отсчёты `f32` в диапазоне [-1, 1], каналы
    /// чередуются внутри кадра. Прежнее содержимое `out` затирается, выделенная под него
    /// память переиспользуется.
    ///
    /// `Ok(false)` — дорожка кончилась. `Ok(true)` с пустым `out` возможен: пакет бывает
    /// нулевой длины, звать дальше.
    ///
    /// Кадры задержки кодера в начале дорожки и набивку в конце отрезает сам декодер:
    /// в `AudioDecoderOptions` признак `gapless` по умолчанию включён. Здесь их трогать
    /// не надо — буфер приходит уже обрезанным.
    pub fn next_frames(&mut self, out: &mut Vec<f32>) -> Result<bool> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(Some(packet)) => packet,
                Ok(None) => {
                    out.clear();
                    return Ok(false);
                }
                Err(err) => {
                    self.consecutive_errors += 1;
                    if self.consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        return Err(anyhow!("слишком много ошибок чтения подряд (последняя: {})", err));
                    }
                    continue;
                }
            };

            if packet.track_id != self.track_id {
                continue;
            }

            let decoded = match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    // Успех — сбрасываем счётчик подряд идущих сбоев
                    self.consecutive_errors = 0;
                    decoded
                }
                Err(err) => {
                    self.consecutive_errors += 1;
                    if self.consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                        return Err(anyhow!("слишком много ошибок декодирования подряд (последняя: {})", err));
                    }
                    self.decoder.reset();
                    continue;
                }
            };

            let spec = decoded.spec();
            if spec.rate() != self.spec.sample_rate
                || spec.channels().count() != usize::from(self.spec.channels)
            {
                return Err(anyhow!(
                    "формат посреди дорожки сменился: было {} Гц и {} каналов, стало {} Гц и {} каналов",
                    self.spec.sample_rate,
                    self.spec.channels,
                    spec.rate(),
                    spec.channels().count()
                ));
            }

            decoded.copy_to_vec_interleaved(out);
            return Ok(true);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;
    use std::io::Cursor;
    use std::time::Duration;

    use super::{PcmSpec, TrackDecoder};

    const RATE: u32 = 44_100;
    const CHANNELS: u16 = 2;
    const FRAMES: usize = 22_050;
    const BITS: u16 = 16;
    /// Шаг 16-битного отсчёта после перевода в f32. С ним же сравнивается расхождение:
    /// wav не сжимает, и кадры обязаны вернуться теми же с точностью до одного шага.
    const STEP: f32 = 1.0 / 32_768.0;

    /// Левый канал — 440 Гц, правый — 110 Гц и вдвое тише. Разные каналы нужны затем,
    /// чтобы перепутанное чередование провалило сравнение, а не прошло незамеченным.
    fn reference_samples() -> Vec<i16> {
        let mut samples = Vec::with_capacity(FRAMES * usize::from(CHANNELS));
        for frame in 0..FRAMES {
            let seconds = frame as f32 / RATE as f32;
            let left = (TAU * 440.0 * seconds).sin() * 0.5;
            let right = (TAU * 110.0 * seconds).sin() * 0.25;
            samples.push((left * f32::from(i16::MAX)).round() as i16);
            samples.push((right * f32::from(i16::MAX)).round() as i16);
        }
        samples
    }

    /// Собирает 44-байтовый заголовок wav и отсчёты за ним. Тест не трогает файловую
    /// систему: `Cursor` над этим вектором уже является источником для symphonia.
    fn wav(samples: &[i16]) -> Vec<u8> {
        let bytes_per_sample = u32::from(BITS / 8);
        let data_len =
            u32::try_from(samples.len()).expect("столько отсчётов тест не делает") * bytes_per_sample;
        let block_align = u32::from(CHANNELS) * bytes_per_sample;

        let mut out = Vec::with_capacity(44 + data_len as usize);
        out.extend_from_slice(b"RIFF");
        out.extend_from_slice(&(36 + data_len).to_le_bytes());
        out.extend_from_slice(b"WAVE");
        out.extend_from_slice(b"fmt ");
        out.extend_from_slice(&16u32.to_le_bytes());
        out.extend_from_slice(&1u16.to_le_bytes());
        out.extend_from_slice(&CHANNELS.to_le_bytes());
        out.extend_from_slice(&RATE.to_le_bytes());
        out.extend_from_slice(&(RATE * block_align).to_le_bytes());
        out.extend_from_slice(&(block_align as u16).to_le_bytes());
        out.extend_from_slice(&BITS.to_le_bytes());
        out.extend_from_slice(b"data");
        out.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            out.extend_from_slice(&sample.to_le_bytes());
        }
        out
    }

    #[test]
    fn wav_decodes_frame_for_frame() {
        let samples = reference_samples();
        let mut decoder = TrackDecoder::open(Box::new(Cursor::new(wav(&samples))), "wav")
            .expect("wav не открылся");

        assert_eq!(
            decoder.spec(),
            PcmSpec {
                sample_rate: RATE,
                channels: CHANNELS,
            }
        );
        assert_eq!(decoder.duration(), Some(Duration::from_millis(500)));

        let mut decoded: Vec<f32> = Vec::new();
        let mut packet: Vec<f32> = Vec::new();
        while decoder.next_frames(&mut packet).expect("декодирование") {
            decoded.extend_from_slice(&packet);
        }

        assert_eq!(
            decoded.len(),
            samples.len(),
            "декодировано отсчётов не столько, сколько записано"
        );
        for (index, (got, want)) in decoded.iter().zip(samples.iter()).enumerate() {
            let want = f32::from(*want) / 32_768.0;
            assert!(
                (*got - want).abs() <= STEP,
                "отсчёт {index}: {got}, ожидался {want}"
            );
        }
    }

    #[test]
    fn seek_lands_where_it_says() {
        let samples = reference_samples();
        let mut decoder = TrackDecoder::open(Box::new(Cursor::new(wav(&samples))), "wav")
            .expect("wav не открылся");

        let actual = decoder
            .seek(Duration::from_millis(250))
            .expect("перемотка удалась");
        assert!(
            actual <= Duration::from_millis(250),
            "точная перемотка не должна проскакивать дальше просьбы, а попала на {actual:?}"
        );

        // Сверяем не с тем, куда просили, а с тем, куда попали: контейнер вправе встать
        // раньше, и место возвращается как раз затем, чтобы по нему считать позицию.
        let frame = (actual.as_secs_f64() * f64::from(RATE)).round() as usize;
        let expected = &samples[frame * usize::from(CHANNELS)..];

        let mut decoded: Vec<f32> = Vec::new();
        let mut packet: Vec<f32> = Vec::new();
        while decoder.next_frames(&mut packet).expect("декодирование") {
            decoded.extend_from_slice(&packet);
        }

        assert_eq!(
            decoded.len(),
            expected.len(),
            "после перемотки должен остаться ровно хвост дорожки"
        );
        for (index, (got, want)) in decoded.iter().zip(expected.iter()).enumerate() {
            let want = f32::from(*want) / 32_768.0;
            assert!(
                (*got - want).abs() <= STEP,
                "отсчёт {index} после перемотки: {got}, ожидался {want}"
            );
        }
    }

    #[test]
    fn garbage_is_not_a_track() {
        let garbage = vec![0x7fu8; 4096];
        assert!(TrackDecoder::open(Box::new(Cursor::new(garbage)), "").is_err());
    }
}
