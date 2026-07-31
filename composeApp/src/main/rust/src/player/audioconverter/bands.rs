pub fn compute_bands(fft_array: Vec<f64>) -> Vec<f64> {
    if fft_array.is_empty() {
        return Vec::new();
    }

    const PART_NUMBER: usize = 32;
    const FREQUENCY_EXPONENT: f64 = 1.5;
    const RELEVANT_DATA_FACTOR: f64 = 0.5;

    let relevant_size = (fft_array.len() as f64 * RELEVANT_DATA_FACTOR) as usize;
    let mut result = Vec::with_capacity(PART_NUMBER);

    for i in 0..PART_NUMBER {
        let start_percent = (i as f64 / PART_NUMBER as f64).powf(FREQUENCY_EXPONENT);
        let end_percent = ((i + 1) as f64 / PART_NUMBER as f64).powf(FREQUENCY_EXPONENT);

        let start_index = (start_percent * relevant_size as f64) as usize;
        let mut end_index = (end_percent * relevant_size as f64) as usize;
        if end_index <= start_index {
            end_index = start_index + 1;
        }

        let mut max_val = 0.0;
        let limit = end_index.min(fft_array.len());
        for j in start_index..limit {
            if fft_array[j] > max_val {
                max_val = fft_array[j];
            }
        }
        result.push(max_val);
    }
    result
}
