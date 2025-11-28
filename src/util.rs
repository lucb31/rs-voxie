use std::{collections::VecDeque, time::Instant};

pub struct SimpleMovingAverage {
    window: VecDeque<f32>,
    period: usize,
    sum: f32,
}

impl SimpleMovingAverage {
    pub fn new(period: usize) -> Self {
        Self {
            window: VecDeque::with_capacity(period),
            period,
            sum: 0.0,
        }
    }

    /// Tracks time elapsed since the instant provided
    pub fn add_elapsed(&mut self, start_time: Instant) -> f32 {
        let tick_time_ns = start_time.elapsed().as_secs_f32() * 1e6;
        self.add(tick_time_ns)
    }

    pub fn add(&mut self, value: f32) -> f32 {
        self.window.push_back(value);
        self.sum += value;

        if self.window.len() > self.period {
            if let Some(removed) = self.window.pop_front() {
                self.sum -= removed;
            }
        }

        self.get()
    }

    pub fn get(&self) -> f32 {
        self.sum / self.window.len() as f32
    }
}
