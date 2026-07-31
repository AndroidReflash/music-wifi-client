use std::array::from_fn;
use std::f32::consts::{SQRT_2, TAU};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// Сколько полос у эквалайзера. Полосы стоят через октаву: слух различает высоту
/// логарифмически, и равный шаг в герцах отдал бы почти все полосы верхам.
pub const EQ_BANDS: usize = 10;

/// Центры полос, Гц: каждая вдвое выше предыдущей, от 31,25 до 16 000.
pub const EQ_FREQUENCIES: [f32; EQ_BANDS] = [
    31.25, 62.5, 125.0, 250.0, 500.0, 1_000.0, 2_000.0, 4_000.0, 8_000.0, 16_000.0,
];

/// Предел подъёма и ослабления полосы, дБ.
pub const EQ_MAX_DB: f32 = 12.0;

/// Добротность фильтра — насколько узко он захватывает частоты вокруг центра. √2 даёт
/// ширину в октаву, то есть ровно шаг между центрами полос.
const OCTAVE_Q: f32 = SQRT_2;

/// Усиления полос, которые пишет управляющий поток и читает колбэк вывода.
///
/// Каждое усиление — битами `f32` в своём атомике: колбэку замок брать нельзя, а атомарного
/// `f32` в языке нет. Счётчик правок говорит колбэку, что пора пересчитать фильтры: сверить
/// одно число дешевле, чем десять на каждом периоде.
pub struct EqSettings {
    gains_db: [AtomicU32; EQ_BANDS],
    version: AtomicU64,
}

impl EqSettings {
    pub fn new() -> Self {
        EqSettings {
            gains_db: from_fn(|_| AtomicU32::new(0.0f32.to_bits())),
            version: AtomicU64::new(0),
        }
    }

    /// Записывает усиления всех полос, дБ, в пределах ±`EQ_MAX_DB`.
    ///
    /// Колбэк может прочитать набор посреди записи — часть полос новые, часть прежние. На
    /// следующем периоде счётчик уже сдвинут, и он дочитает остальное.
    pub fn set(&self, gains_db: [f32; EQ_BANDS]) {
        for (held, gain) in self.gains_db.iter().zip(gains_db) {
            held.store(gain.clamp(-EQ_MAX_DB, EQ_MAX_DB).to_bits(), Ordering::Relaxed);
        }
        self.version.fetch_add(1, Ordering::Release);
    }

    fn version(&self) -> u64 {
        self.version.load(Ordering::Acquire)
    }

    fn gains_db(&self) -> [f32; EQ_BANDS] {
        from_fn(|band| f32::from_bits(self.gains_db[band].load(Ordering::Relaxed)))
    }
}

/// Коэффициенты одного пикового фильтра, уже поделённые на `a0`.
#[derive(Clone, Copy)]
struct Coefficients {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl Coefficients {
    /// Пиковый фильтр из сборника формул Роберта Бристоу-Джонсона (RBJ Audio EQ Cookbook):
    /// поднимает или опускает окрестность `frequency` на `gain_db`, остальное пропускает как
    /// есть. На нуле децибел числитель равен знаменателю, и фильтр ничего не меняет.
    ///
    /// Частота входит только долей частоты дискретизации, поэтому коэффициенты верны лишь
    /// для той частоты, под которую посчитаны.
    ///
    /// Полоса, которой при этой частоте дискретизации не бывает (см. [`fits`]), пропускает звук
    /// как есть: по формуле её фильтр вышел бы неустойчивым.
    fn peaking(frequency: f32, gain_db: f32, sample_rate: u32) -> Self {
        if !fits(frequency, sample_rate) {
            return Coefficients {
                b0: 1.0,
                b1: 0.0,
                b2: 0.0,
                a1: 0.0,
                a2: 0.0,
            };
        }
        let amplitude = 10f32.powf(gain_db / 40.0);
        let omega = TAU * frequency / sample_rate as f32;
        let alpha = omega.sin() / (2.0 * OCTAVE_Q);
        let cos = omega.cos();

        let a0 = 1.0 + alpha / amplitude;
        Coefficients {
            b0: (1.0 + alpha * amplitude) / a0,
            b1: -2.0 * cos / a0,
            b2: (1.0 - alpha * amplitude) / a0,
            a1: -2.0 * cos / a0,
            a2: (1.0 - alpha / amplitude) / a0,
        }
    }
}

/// Есть ли такая частота в звуке с этой частотой дискретизации. Выше половины частоты
/// дискретизации звука не бывает. Центр фильтра между половиной и полной частотой даёт в
/// формуле отрицательный синус: память фильтра растёт с каждым отсчётом, и звук уходит в рёв —
/// так полоса 16 кГц ведёт себя на 22 050 и 24 000 Гц. Центр ровно на половине или на кратном
/// частоты попадает на ноль синуса, и что выйдет, решает погрешность округления, — так на 8 и
/// 16 кГц, частотах гарнитуры в режиме разговора.
fn fits(frequency: f32, sample_rate: u32) -> bool {
    frequency < sample_rate as f32 / 2.0
}

/// Память одного фильтра на одном канале — транспонированная вторая прямая форма, два числа.
#[derive(Clone, Copy, Default)]
struct State {
    z1: f32,
    z2: f32,
}

/// Эквалайзер на стороне колбэка: фильтры и их память. Живёт внутри колбэка одного потока
/// cpal, поэтому частота дискретизации у него постоянна, а при смене устройства он
/// создаётся заново вместе с потоком.
pub struct Equalizer {
    sample_rate: u32,
    channels: usize,
    coefficients: [Coefficients; EQ_BANDS],
    /// Память фильтров: `channels` подряд идущих наборов по `EQ_BANDS`.
    states: Vec<State>,
    /// Множитель после фильтров — ослабление на самый большой подъём, чтобы поднятая
    /// полоса не выводила отсчёты за ±1.
    preamp: f32,
    /// Какую правку настроек фильтры уже отражают.
    seen: Option<u64>,
}

impl Equalizer {
    /// Память выделяется здесь, до колбэка: в нём самом выделять нельзя.
    pub fn new(sample_rate: u32, channels: u16) -> Self {
        let channels = usize::from(channels);
        Equalizer {
            sample_rate,
            channels,
            coefficients: from_fn(|band| {
                Coefficients::peaking(EQ_FREQUENCIES[band], 0.0, sample_rate)
            }),
            states: vec![State::default(); channels * EQ_BANDS],
            preamp: 1.0,
            seen: None,
        }
    }

    /// Пересчитывает фильтры, если настройки менялись с прошлого раза. Память фильтров не
    /// трогает: звук идёт дальше, и обнулённая память дала бы щелчок на каждом сдвиге ползунка.
    pub fn sync(&mut self, settings: &EqSettings) {
        let version = settings.version();
        if self.seen == Some(version) {
            return;
        }
        self.seen = Some(version);

        let gains_db = settings.gains_db();
        for (band, gain_db) in gains_db.iter().enumerate() {
            self.coefficients[band] =
                Coefficients::peaking(EQ_FREQUENCIES[band], *gain_db, self.sample_rate);
        }
        // Подъём полосы, которой нет, ничего не поднимает, и ослаблять под него звук незачем.
        let boost_db = gains_db
            .iter()
            .zip(EQ_FREQUENCIES)
            .filter(|(_, frequency)| fits(*frequency, self.sample_rate))
            .fold(0.0f32, |most, (gain, _)| most.max(*gain));
        self.preamp = 10f32.powf(-boost_db / 20.0);
    }

    /// Забывает прошлое звучание — на перемотке и выбросе кольца: то, что шло дальше, не
    /// продолжение того, что было, и хвост прежнего звука в памяти фильтров был бы щелчком.
    pub fn reset(&mut self) {
        self.states.fill(State::default());
    }

    /// Пропускает кадры через все полосы. `samples` — чередующиеся каналы, как их отдаёт
    /// кольцо.
    pub fn process(&mut self, samples: &mut [f32]) {
        for frame in samples.chunks_exact_mut(self.channels) {
            for (channel, sample) in frame.iter_mut().enumerate() {
                let states = &mut self.states[channel * EQ_BANDS..(channel + 1) * EQ_BANDS];
                let mut value = *sample;
                for (c, state) in self.coefficients.iter().zip(states.iter_mut()) {
                    let out = c.b0 * value + state.z1;
                    state.z1 = c.b1 * value - c.a1 * out + state.z2;
                    state.z2 = c.b2 * value - c.a2 * out;
                    flush_subnormal(&mut state.z1);
                    flush_subnormal(&mut state.z2);
                    value = out;
                }
                *sample = value * self.preamp;
            }
        }
    }
}

/// Сбрасывает в ноль число, которое ушло ниже нормального диапазона `f32`.
///
/// После обрыва звука память фильтра затухает к нулю и по пути проходит через такие числа
/// (денормали). Процессор считает их особым медленным путём, и колбэк, который до этого
/// укладывался в срок, начинает опаздывать как раз в тишине. Слышимого в них ничего нет:
/// это меньше 10⁻³⁸.
fn flush_subnormal(value: &mut f32) {
    if value.is_subnormal() {
        *value = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::{EqSettings, Equalizer, EQ_BANDS, EQ_FREQUENCIES, EQ_MAX_DB};

    const RATE: u32 = 48_000;

    /// Моно синус длиной в секунду.
    fn sine(frequency: f32) -> Vec<f32> {
        (0..RATE as usize)
            .map(|i| (TAU * frequency * i as f32 / RATE as f32).sin())
            .collect()
    }

    /// Среднеквадратичное второй половины, в долях от чистого синуса: к ней фильтры давно
    /// вошли в установившийся режим. Не пик — на высоких частотах отсчёты не попадают на
    /// вершину волны, и пик синуса 8 кГц при 48 кГц выходит на 1,25 дБ ниже настоящего.
    fn settled_level(samples: &[f32]) -> f32 {
        let tail = &samples[samples.len() / 2..];
        let mean_square = tail.iter().map(|it| it * it).sum::<f32>() / tail.len() as f32;
        (2.0 * mean_square).sqrt()
    }

    fn db(ratio: f32) -> f32 {
        20.0 * ratio.log10()
    }

    fn equalized(gains_db: [f32; EQ_BANDS], mut samples: Vec<f32>) -> Vec<f32> {
        let settings = EqSettings::new();
        settings.set(gains_db);
        let mut equalizer = Equalizer::new(RATE, 1);
        equalizer.sync(&settings);
        equalizer.process(&mut samples);
        samples
    }

    #[test]
    fn flat_settings_leave_sound_unchanged() {
        let input = sine(440.0);
        let output = equalized([0.0; EQ_BANDS], input.clone());
        for (a, b) in input.iter().zip(&output) {
            assert!((a - b).abs() < 1e-5, "{a} против {b}");
        }
    }

    #[test]
    fn boosted_band_stays_at_full_scale_and_the_rest_drops() {
        let mut gains = [0.0; EQ_BANDS];
        gains[5] = EQ_MAX_DB;

        // Центр поднятой полосы: подъём и предусилитель гасят друг друга.
        let boosted = settled_level(&equalized(gains, sine(EQ_FREQUENCIES[5])));
        assert!(db(boosted).abs() < 0.1, "центр полосы: {} дБ", db(boosted));

        // Далеко от неё звучит только предусилитель.
        let far = settled_level(&equalized(gains, sine(EQ_FREQUENCIES[0])));
        assert!((db(far) + EQ_MAX_DB).abs() < 0.3, "далёкая полоса: {} дБ", db(far));
    }

    #[test]
    fn cut_band_drops_and_needs_no_preamp() {
        let mut gains = [0.0; EQ_BANDS];
        gains[3] = -EQ_MAX_DB;

        let cut = settled_level(&equalized(gains, sine(EQ_FREQUENCIES[3])));
        assert!((db(cut) + EQ_MAX_DB).abs() < 0.3, "центр полосы: {} дБ", db(cut));

        let far = settled_level(&equalized(gains, sine(EQ_FREQUENCIES[8])));
        assert!(db(far).abs() < 0.1, "далёкая полоса: {} дБ", db(far));
    }

    #[test]
    fn gains_are_clamped() {
        let mut gains = [0.0; EQ_BANDS];
        gains[5] = 40.0;
        let far = settled_level(&equalized(gains, sine(EQ_FREQUENCIES[0])));
        assert!((db(far) + EQ_MAX_DB).abs() < 0.3, "далёкая полоса: {} дБ", db(far));
    }

    #[test]
    fn sync_picks_up_a_later_change() {
        let settings = EqSettings::new();
        let mut equalizer = Equalizer::new(RATE, 1);
        equalizer.sync(&settings);

        let mut gains = [0.0; EQ_BANDS];
        gains[0] = -EQ_MAX_DB;
        settings.set(gains);
        equalizer.sync(&settings);

        let mut samples = sine(EQ_FREQUENCIES[0]);
        equalizer.process(&mut samples);
        let cut = settled_level(&samples);
        assert!((db(cut) + EQ_MAX_DB).abs() < 0.3, "{} дБ", db(cut));
    }

    #[test]
    fn channels_are_filtered_apart() {
        let mut gains = [0.0; EQ_BANDS];
        gains[5] = -EQ_MAX_DB;
        let settings = EqSettings::new();
        settings.set(gains);
        let mut equalizer = Equalizer::new(RATE, 2);
        equalizer.sync(&settings);

        // Левый канал — центр ослабленной полосы, правый — тишина.
        let mut samples: Vec<f32> = sine(EQ_FREQUENCIES[5])
            .into_iter()
            .flat_map(|it| [it, 0.0])
            .collect();
        equalizer.process(&mut samples);

        let right: Vec<f32> = samples.iter().skip(1).step_by(2).copied().collect();
        assert!(right.iter().all(|it| *it == 0.0), "в правый канал просочился левый");
    }

    #[test]
    fn band_above_half_the_rate_passes_sound_as_is() {
        // 22 050 Гц: полоса 16 кГц выше половины частоты, но ниже полной — там синус в
        // формуле отрицательный, и фильтр по формуле неустойчив.
        const LOW_RATE: u32 = 22_050;
        let mut gains = [0.0; EQ_BANDS];
        gains[9] = EQ_MAX_DB;
        let settings = EqSettings::new();
        settings.set(gains);
        let mut equalizer = Equalizer::new(LOW_RATE, 1);
        equalizer.sync(&settings);

        let mut samples: Vec<f32> = (0..LOW_RATE as usize)
            .map(|i| (TAU * EQ_FREQUENCIES[5] * i as f32 / LOW_RATE as f32).sin())
            .collect();
        equalizer.process(&mut samples);

        assert!(samples.iter().all(|it| it.is_finite()), "фильтр разошёлся");
        let level = settled_level(&samples);
        assert!(db(level).abs() < 0.1, "{} дБ", db(level));
    }

    #[test]
    fn memory_decays_to_zero_not_to_subnormals() {
        let mut gains = [0.0; EQ_BANDS];
        gains[9] = EQ_MAX_DB;
        let settings = EqSettings::new();
        settings.set(gains);
        let mut equalizer = Equalizer::new(RATE, 1);
        equalizer.sync(&settings);

        let mut loud = sine(EQ_FREQUENCIES[9]);
        equalizer.process(&mut loud);
        // Минута тишины: за неё память любого фильтра затухает за нижний край `f32`.
        let mut silence = vec![0.0; 60 * RATE as usize];
        equalizer.process(&mut silence);

        for state in &equalizer.states {
            assert!(!state.z1.is_subnormal() && !state.z2.is_subnormal());
        }
        assert!(equalizer.states.iter().all(|it| it.z1 == 0.0 && it.z2 == 0.0));
    }

    #[test]
    fn reset_forgets_previous_sound() {
        let mut gains = [0.0; EQ_BANDS];
        gains[0] = EQ_MAX_DB;
        let settings = EqSettings::new();
        settings.set(gains);
        let mut equalizer = Equalizer::new(RATE, 1);
        equalizer.sync(&settings);

        let mut loud = sine(EQ_FREQUENCIES[0]);
        equalizer.process(&mut loud);
        equalizer.reset();

        let mut silence = vec![0.0; 64];
        equalizer.process(&mut silence);
        assert!(silence.iter().all(|it| *it == 0.0), "после сброса звучит хвост");
    }
}
