pub struct FrequencyBinner {
    num_bars: usize,
    sample_rate: f32,
    fft_size: usize,
}

impl FrequencyBinner {
    pub fn new(num_bars: usize, sample_rate: f32, fft_size: usize) -> Self {
        FrequencyBinner {
            num_bars,
            sample_rate,
            fft_size,
        }
    }

    pub fn calculate_bins(&self, fft_magnitudes: &[f32]) -> Vec<f32> {
        let mut bars = vec![0.0; self.num_bars];
        let num_buckets = fft_magnitudes.len();

        let min_freq = 20.0f32;
        let max_freq = 20000.0f32;

        let min_log = min_freq.log10();
        let max_log = max_freq.log10();
        let log_step = (max_log - min_log) / self.num_bars as f32;

        for bar_idx in 0..self.num_bars {
            let low_log = min_log + bar_idx as f32 * log_step;
            let high_log = min_log + (bar_idx + 1) as f32 * log_step;

            let low_freq = 10.0f32.powf(low_log);
            let high_freq = 10.0f32.powf(high_log);

            let low_bin = (low_freq / self.sample_rate * self.fft_size as f32).floor() as usize;
            let high_bin = (high_freq / self.sample_rate * self.fft_size as f32).floor() as usize;

            let start_bin = low_bin.min(num_buckets);
            let end_bin = (high_bin + 1).max(start_bin + 1).min(num_buckets);

            let mut sum = 0.0;
            let mut count = 0;

            for bin in start_bin..end_bin {
                sum += fft_magnitudes[bin];
                count += 1;
            }

            let mut energy = if count > 0 { sum / count as f32 } else { 0.0 };

            energy = energy.powf(0.5);
            bars[bar_idx] = energy;
        }

        return bars;
    }
}
