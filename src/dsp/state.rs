use std::sync::{Arc, Mutex};

#[derive(Clone, PartialEq)]
pub enum AppView {
    SelectDevice,
    Visualizer,
}

#[derive(Clone)]
pub struct AppState {
    pub current_view: Arc<Mutex<AppView>>,
    pub num_bars: Arc<Mutex<usize>>,
    pub response: Arc<Mutex<f32>>,
    pub current_bins: Arc<Mutex<Vec<f32>>>,
}

impl AppState {
    pub fn new(initial_bars: usize) -> Self {
        Self {
            current_view: Arc::new(Mutex::new(AppView::SelectDevice)),
            num_bars: Arc::new(Mutex::new(initial_bars)),
            response: Arc::new(Mutex::new(0.20)),
            current_bins: Arc::new(Mutex::new(vec![0.0; initial_bars])),
        }
    }

    pub fn update_bins(&self, bins: Vec<f32>) {
        if let Ok(mut lock) = self.current_bins.lock() {
            *lock = bins;
        }
    }

    pub fn get_bins(&self) -> Vec<f32> {
        if let Ok(lock) = self.current_bins.lock() {
            lock.clone()
        } else {
            Vec::new()
        }
    }

    pub fn get_view(&self) -> AppView {
        if let Ok(lock) = self.current_view.lock() {
            lock.clone()
        } else {
            AppView::SelectDevice
        }
    }

    pub fn set_view(&self, view: AppView) {
        if let Ok(mut lock) = self.current_view.lock() {
            *lock = view;
        }
    }

    pub fn get_num_bars(&self) -> usize {
        if let Ok(lock) = self.num_bars.lock() {
            *lock
        } else {
            32
        }
    }

    pub fn adjust_bars(&self, delta: i32) {
        if let Ok(mut lock) = self.num_bars.lock() {
            let current = *lock as i32;
            let new_val = (current + delta).clamp(4, 128);
            *lock = new_val as usize;
            if let Ok(mut bins) = self.current_bins.lock() {
                bins.resize(new_val as usize, 0.0);
            }
        }
    }

    pub fn get_response(&self) -> f32 {
        self.response
            .lock()
            .map(|response| *response)
            .unwrap_or(0.20)
    }

    pub fn adjust_response(&self, delta: f32) {
        if let Ok(mut response) = self.response.lock() {
            *response = (*response + delta).clamp(0.05, 1.0);
        }
    }
}
