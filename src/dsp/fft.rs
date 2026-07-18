use rustfft::{FftPlanner, num_complex::Complex};

pub struct FftProcessor {
    fft_size: usize,
    planner: FftPlanner<f32>,
}

impl FftProcessor {
    pub fn new(fft_size: usize) -> Self {
        Self {
            fft_size,
            planner: FftPlanner::new(),
        }
    }

    pub fn process(&mut self, samples: &[f32]) -> Vec<f32> {
        if samples.len() < self.fft_size {
            return vec![0.0; self.fft_size / 2];
        }

        let mut buffer: Vec<Complex<f32>> = samples[..self.fft_size]
            .iter()
            .enumerate()
            .map(|(i, &sample)| {
                let window = 0.5
                    * (1.0
                        - (2.0 * std::f32::consts::PI * i as f32 / (self.fft_size - 1) as f32)
                            .cos());
                Complex::new(sample * window, 0.0)
            })
            .collect();

        let fft = self.planner.plan_fft_forward(self.fft_size);
        fft.process(&mut buffer);

        let mut magnitudes = Vec::with_capacity(self.fft_size / 2);
        for i in 0..(self.fft_size / 2) {
            magnitudes.push(buffer[i].norm());
        }

        magnitudes
    }
}
