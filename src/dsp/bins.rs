use std::ops::Range;

pub struct FrequencyBinner {
    ranges: Vec<Range<usize>>,
}

impl FrequencyBinner {
    pub fn new(num_bars: usize, sample_rate: f32, fft_size: usize) -> Self {
        let spectrum_len = fft_size / 2;
        let bin_width = sample_rate / fft_size as f32;
        let min_frequency = 20.0;
        let max_frequency = 20_000.0f32.min(sample_rate / 2.0);
        let first_bin = (min_frequency / bin_width).ceil() as usize;
        let end_bin = (max_frequency / bin_width).ceil() as usize;
        let first_bin = first_bin.clamp(1, spectrum_len);
        let end_bin = end_bin.clamp(first_bin, spectrum_len);
        let range_bins = end_bin.saturating_sub(first_bin);
        let bar_count = num_bars.min(range_bins).max(1);
        let min_log = min_frequency.log10();
        let max_log = max_frequency.max(min_frequency).log10();
        let mut ranges = Vec::with_capacity(bar_count);
        let mut start = first_bin;

        for bar_index in 0..bar_count {
            let end = if bar_index + 1 == bar_count {
                end_bin
            } else {
                let progress = (bar_index + 1) as f32 / bar_count as f32;
                let target_frequency = 10.0f32.powf(min_log + (max_log - min_log) * progress);
                let target_bin = (target_frequency / bin_width).round() as usize;
                target_bin.clamp(start + 1, end_bin.saturating_sub(bar_count - bar_index - 1))
            };
            ranges.push(start..end);
            start = end;
        }

        Self { ranges }
    }

    pub fn num_bars(&self) -> usize {
        self.ranges.len()
    }

    pub fn calculate_bins(&self, fft_magnitudes: &[f32]) -> Vec<f32> {
        self.ranges
            .iter()
            .map(|range| {
                let mut sum_squares = 0.0;
                let mut count = 0;
                for &magnitude in fft_magnitudes
                    .get(range.start..range.end.min(fft_magnitudes.len()))
                    .unwrap_or_default()
                {
                    sum_squares += magnitude * magnitude;
                    count += 1;
                }
                if count == 0 {
                    0.0
                } else {
                    (sum_squares / count as f32).sqrt()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::FrequencyBinner;

    #[test]
    fn logarithmic_ranges_cover_the_audible_spectrum_without_gaps() {
        let binner = FrequencyBinner::new(10, 48_000.0, 2_048);

        assert_eq!(binner.num_bars(), 10);
        assert!(
            binner
                .ranges
                .windows(2)
                .all(|ranges| ranges[0].end == ranges[1].start)
        );
        assert!(binner.ranges.iter().all(|range| !range.is_empty()));
    }

    #[test]
    fn fewer_bars_aggregate_all_input_energy() {
        let binner = FrequencyBinner::new(10, 48_000.0, 2_048);
        let mut spectrum = vec![0.0; 1_024];
        spectrum[1] = 1.0;
        spectrum[800] = 1.0;
        let bins = binner.calculate_bins(&spectrum);

        assert!(bins.first().is_some_and(|value| *value > 0.0));
        assert!(bins.last().is_some_and(|value| *value > 0.0));
    }
}
