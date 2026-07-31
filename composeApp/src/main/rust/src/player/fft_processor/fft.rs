use std::sync::{Arc, OnceLock};

use num_complex::Complex;
use rustfft::{Fft, FftPlanner};

/// Сколько значений уходит в одно преобразование. Значения берутся из кольца вывода
/// подряд, вперемешку по каналам, — так же, как их брали из буфера ExoPlayer до
/// переезда на Rust. Каналы не усредняются намеренно: усреднение гасит то, что в них в
/// противофазе, и картина выходит другой.
pub const FFT_SIZE: usize = 1024;

/// План преобразования: таблицы поворотных множителей для [`FFT_SIZE`] точек. Считать их
/// заново на каждый вызов дороже самого преобразования, а размер окна всегда один.
fn plan() -> &'static Arc<dyn Fft<f64>> {
    static PLAN: OnceLock<Arc<dyn Fft<f64>>> = OnceLock::new();
    PLAN.get_or_init(|| FftPlanner::new().plan_fft_forward(FFT_SIZE))
}

/// Takes normalized f64 audio samples, performs FFT analysis,
/// and returns the first half of the spectrum amplitudes.
pub fn analyze_samples(samples: &[f64]) -> Vec<f64> {
    if samples.is_empty() {
        return Vec::new();
    }

    let mut buffer: Vec<Complex<f64>> = Vec::with_capacity(FFT_SIZE);

    for &sample in samples.iter().take(FFT_SIZE) {
        buffer.push(Complex::new(sample, 0.0));
    }

    while buffer.len() < FFT_SIZE {
        buffer.push(Complex::new(0.0, 0.0));
    }

    plan().process(&mut buffer);

    let norm_factor = 1.0 / (FFT_SIZE as f64).sqrt();
    let half_size = FFT_SIZE / 2;
    let mut amplitudes = Vec::with_capacity(half_size);

    for complex in &buffer[..half_size] {
        amplitudes.push(complex.norm() * norm_factor);
    }

    amplitudes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        assert!(analyze_samples(&[]).is_empty());
    }

    #[test]
    fn test_fft_length() {
        // Короче окна — остаток дополняется нулями, длина ответа от этого не меняется.
        let amplitudes = analyze_samples(&vec![0.0; 100]);
        assert_eq!(amplitudes.len(), FFT_SIZE / 2);
    }
}
