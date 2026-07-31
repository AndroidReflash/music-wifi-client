use anyhow::{anyhow, Context, Result};
use audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Indexing, Resampler};

use crate::player::decoder::PcmSpec;

/// Сколько кадров ресемплер берёт за один заход. 1024 кадра — это 23 мс при 44100 Гц:
/// порция меньше делает срез противомаскировочного фильтра грубее, порция больше растит
/// задержку, а для музыки задержка значения не имеет.
const RESAMPLER_CHUNK: usize = 1024;

/// Как раскладывать каналы дорожки по каналам устройства.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChannelMap {
    /// Каналов поровну — кадры идут как есть.
    AsIs,
    /// Моно на несколько каналов — один и тот же отсчёт уходит во все.
    Duplicate,
}

/// Приводит кадры дорожки к тому, что принимает устройство: частоту — ресемплером,
/// каналы — раскладкой. Между ними накопитель: ресемплер берёт кадры порциями строго
/// определённой длины, а декодер отдаёт пакеты какой придётся.
///
/// Конвертер живёт одну дорожку: `drain` заканчивает её, и на следующую нужен новый.
pub struct FrameConverter {
    input: PcmSpec,
    output: PcmSpec,
    map: ChannelMap,
    /// `None`, когда частоты совпадают и пересчитывать нечего.
    resampler: Option<Fft<f32>>,
    /// Кадры дорожки, не набравшие порцию ресемплера.
    staged: Vec<f32>,
    /// Выход ресемплера: каналов столько же, сколько в дорожке.
    resampled: Vec<f32>,
    /// Ресемплер начинает с тишины длиной в свою задержку. Это не звук дорожки,
    /// и столько кадров с начала выбрасывается.
    skip: usize,
    frames_in: u64,
    frames_out: u64,
}

impl FrameConverter {
    pub fn new(input: PcmSpec, output: PcmSpec) -> Result<Self> {
        if input.channels == 0 || output.channels == 0 {
            return Err(anyhow!(
                "каналов должно быть хотя бы по одному: в дорожке {}, у устройства {}",
                input.channels,
                output.channels
            ));
        }

        let map = if input.channels == output.channels {
            ChannelMap::AsIs
        } else if input.channels == 1 {
            ChannelMap::Duplicate
        } else {
            return Err(anyhow!(
                "в дорожке {} каналов, устройство принимает {} — сведение каналов не написано",
                input.channels,
                output.channels
            ));
        };

        let (resampler, resampled, skip) = if input.sample_rate == output.sample_rate {
            (None, Vec::new(), 0)
        } else {
            let resampler = Fft::<f32>::new(
                input.sample_rate as usize,
                output.sample_rate as usize,
                RESAMPLER_CHUNK,
                usize::from(input.channels),
                FixedSync::Input,
            )
            .context("ресемплер не построился")?;
            let skip = resampler.output_delay();
            let resampled = vec![0.0; resampler.output_frames_max() * usize::from(input.channels)];
            (Some(resampler), resampled, skip)
        };

        Ok(FrameConverter {
            input,
            output,
            map,
            resampler,
            staged: Vec::new(),
            resampled,
            skip,
            frames_in: 0,
            frames_out: 0,
        })
    }

    /// Принимает кадры от декодера и дописывает в `out` готовые для устройства.
    /// За вызов может не выдать ничего: ресемплер ждёт полную порцию.
    pub fn push(&mut self, frames: &[f32], out: &mut Vec<f32>) -> Result<()> {
        let channels_in = usize::from(self.input.channels);
        if frames.len() % channels_in != 0 {
            return Err(anyhow!(
                "в пакете {} отсчётов — это не целое число кадров по {} канала(ов)",
                frames.len(),
                channels_in
            ));
        }
        self.frames_in += (frames.len() / channels_in) as u64;

        if self.resampler.is_none() {
            self.frames_out += (frames.len() / channels_in) as u64;
            write_frames(frames, self.map, usize::from(self.output.channels), out);
            return Ok(());
        }

        self.staged.extend_from_slice(frames);
        self.pump(false, out)
    }

    /// Забывает накопленное: после перемотки прежние кадры к новому месту отношения не
    /// имеют, и счёт кадров начинается заново.
    pub fn reset(&mut self) {
        self.staged.clear();
        self.frames_in = 0;
        self.frames_out = 0;
        if let Some(resampler) = self.resampler.as_mut() {
            resampler.reset();
            self.skip = resampler.output_delay();
        }
    }

    /// Заканчивает дорожку: доводит до `out` и остаток накопителя, и кадры, застрявшие
    /// внутри ресемплера. После вызова в накопителе не остаётся ничего.
    pub fn drain(&mut self, out: &mut Vec<f32>) -> Result<()> {
        self.pump(true, out)
    }

    /// Гоняет ресемплер порциями. `tail` — конец дорожки: тогда неполная порция
    /// добивается тишиной, а лишние кадры на выходе отрезаются.
    fn pump(&mut self, tail: bool, out: &mut Vec<f32>) -> Result<()> {
        let channels_in = usize::from(self.input.channels);
        let channels_out = usize::from(self.output.channels);
        let map = self.map;
        let expected = expected_frames(
            self.frames_in,
            self.input.sample_rate,
            self.output.sample_rate,
        )?;
        let capacity_frames = self.resampled.len() / channels_in;

        let Some(resampler) = self.resampler.as_mut() else {
            return Ok(());
        };

        loop {
            let staged_frames = self.staged.len() / channels_in;
            let needed = resampler.input_frames_next();
            if staged_frames < needed && !tail {
                return Ok(());
            }
            if tail && self.frames_out >= expected {
                return Ok(());
            }

            let mut indexing = Indexing::new();
            if staged_frames < needed {
                indexing = indexing.partial_len(staged_frames);
            }

            let input = InterleavedSlice::new(&self.staged, channels_in, staged_frames.min(needed))
                .map_err(|err| anyhow!("вход ресемплера: {}", err))?;
            let mut output =
                InterleavedSlice::new_mut(&mut self.resampled, channels_in, capacity_frames)
                    .map_err(|err| anyhow!("выход ресемплера: {}", err))?;
            let (consumed, produced) = resampler
                .process_into_buffer(&input, &mut output, Some(&indexing))
                .map_err(|err| anyhow!("пересчёт частоты: {}", err))?;

            let skipped = self.skip.min(produced);
            self.skip -= skipped;
            let usable_frames = produced - skipped;
            write_frames(
                &self.resampled[skipped * channels_in..produced * channels_in],
                map,
                channels_out,
                out,
            );
            self.frames_out += usable_frames as u64;

            if tail && self.frames_out > expected {
                let extra = (self.frames_out - expected) as usize * channels_out;
                out.truncate(out.len() - extra);
                self.frames_out = expected;
            }

            // На неполной порции ресемплер сообщает длину полной, а кадров взял столько,
            // сколько было: недостающее он дополнил тишиной сам.
            let consumed = consumed.min(staged_frames);
            self.staged.drain(..consumed * channels_in);
        }
    }
}

/// Раскладывает кадры по каналам устройства. Раскладка выбрана один раз в
/// `FrameConverter::new`, здесь остаётся только её применить.
fn write_frames(frames: &[f32], map: ChannelMap, channels_out: usize, out: &mut Vec<f32>) {
    match map {
        ChannelMap::AsIs => out.extend_from_slice(frames),
        ChannelMap::Duplicate => {
            out.reserve(frames.len() * channels_out);
            for sample in frames {
                for _ in 0..channels_out {
                    out.push(*sample);
                }
            }
        }
    }
}

/// Сколько кадров должно получиться из `frames_in` при переходе с одной частоты на
/// другую. Остаток отбрасывается: разница не больше одного кадра.
fn expected_frames(frames_in: u64, rate_in: u32, rate_out: u32) -> Result<u64> {
    let frames = u128::from(frames_in) * u128::from(rate_out) / u128::from(rate_in);
    u64::try_from(frames).map_err(|_| anyhow!("кадров на выходе {}, столько не считается", frames))
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::{FrameConverter, PcmSpec};

    /// Частота, на которой собраны проверки. Совпадает с частотой обычной дорожки,
    /// но здесь важна только сама по себе: конвертер частоту не выбирает.
    const RATE: u32 = 44_100;

    fn stereo(sample_rate: u32) -> PcmSpec {
        PcmSpec {
            sample_rate,
            channels: 2,
        }
    }

    #[test]
    fn matching_format_passes_frames_through() {
        let mut converter = FrameConverter::new(stereo(RATE), stereo(RATE)).expect("конвертер");
        let mut out = Vec::new();
        converter
            .push(&[0.1, -0.1, 0.2, -0.2], &mut out)
            .expect("кадры приняты");
        converter.drain(&mut out).expect("хвост дорожки");
        assert_eq!(out, vec![0.1, -0.1, 0.2, -0.2]);
    }

    #[test]
    fn mono_goes_into_both_channels() {
        let mono = PcmSpec {
            sample_rate: RATE,
            channels: 1,
        };
        let mut converter = FrameConverter::new(mono, stereo(RATE)).expect("конвертер");
        let mut out = Vec::new();
        converter.push(&[0.5, -0.25], &mut out).expect("кадры приняты");
        assert_eq!(out, vec![0.5, 0.5, -0.25, -0.25]);
    }

    #[test]
    fn six_channels_have_nowhere_to_go() {
        let surround = PcmSpec {
            sample_rate: RATE,
            channels: 6,
        };
        assert!(FrameConverter::new(surround, stereo(RATE)).is_err());
    }

    #[test]
    fn resampling_keeps_length_and_tone() {
        const TONE_HZ: f32 = 1_000.0;
        const AMPLITUDE: f32 = 0.5;
        const OUT_RATE: u32 = 48_000;

        let mut converter =
            FrameConverter::new(stereo(RATE), stereo(OUT_RATE)).expect("конвертер");
        let mut out = Vec::new();

        // Секунда тона, пакетами по 500 кадров: в порцию ресемплера они не попадают
        // ровно, и это ближе к тому, как отдаёт кадры настоящий декодер.
        let total_frames = RATE as usize;
        let mut packet = Vec::new();
        for frame in 0..total_frames {
            let seconds = frame as f32 / RATE as f32;
            let value = (TAU * TONE_HZ * seconds).sin() * AMPLITUDE;
            packet.push(value);
            packet.push(value);
            if packet.len() == 500 * 2 || frame + 1 == total_frames {
                converter.push(&packet, &mut out).expect("кадры приняты");
                packet.clear();
            }
        }
        converter.drain(&mut out).expect("хвост дорожки");

        assert_eq!(
            out.len(),
            OUT_RATE as usize * 2,
            "секунда звука обязана остаться секундой"
        );

        // Начало — не тишина: задержку ресемплера выбросили, а не отдали как звук.
        let head = out[..200].iter().fold(0.0f32, |peak, value| peak.max(value.abs()));
        assert!(head > 0.05, "первые кадры пустые, пик {head}");

        // Тон остался тем же: считаем смены знака в середине, подальше от краёв.
        // У синуса их ровно два на период, то есть 2 * 1000 Гц * 44000/48000 с.
        let middle: Vec<f32> = out[2_000 * 2..46_000 * 2].iter().step_by(2).copied().collect();
        let crossings = middle
            .windows(2)
            .filter(|pair| (pair[0] < 0.0) != (pair[1] < 0.0))
            .count();
        assert!(
            (1_830..=1_837).contains(&crossings),
            "смен знака {crossings}, а у тона 1000 Гц их около 1833"
        );

        let peak = middle.iter().fold(0.0f32, |peak, value| peak.max(value.abs()));
        assert!(
            (AMPLITUDE - peak).abs() < 0.05,
            "амплитуда после пересчёта {peak} вместо {AMPLITUDE}"
        );
    }
}
