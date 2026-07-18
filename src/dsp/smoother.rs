pub struct SpectrumSmoother {
    values: Vec<f32>,
    response: f32,
}

impl SpectrumSmoother {
    pub fn new(response: f32) -> Self {
        Self {
            values: Vec::new(),
            response: response.clamp(0.05, 1.0),
        }
    }

    pub fn set_response(&mut self, response: f32) {
        self.response = response.clamp(0.05, 1.0);
    }

    pub fn process(&mut self, target: Vec<f32>) -> Vec<f32> {
        if self.values.len() != target.len() {
            self.values = target;
            return self.values.clone();
        }

        for (value, target) in self.values.iter_mut().zip(target) {
            *value += (target - *value) * self.response;
        }

        self.values.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::SpectrumSmoother;

    #[test]
    fn response_controls_the_distance_moved_toward_the_target() {
        let mut smoother = SpectrumSmoother::new(0.25);
        assert_eq!(smoother.process(vec![0.0]), vec![0.0]);
        assert_eq!(smoother.process(vec![1.0]), vec![0.25]);
    }
}
