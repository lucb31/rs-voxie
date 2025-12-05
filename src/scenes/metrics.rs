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
pub struct SceneMetrics {
    pub sma_dt: SimpleMovingAverage,
    pub sma_render_loop: SimpleMovingAverage,
    pub sma_render_time: SimpleMovingAverage,
    pub sma_swap_time: SimpleMovingAverage,
    pub sma_tick_time: SimpleMovingAverage,
}

impl SceneMetrics {
    pub fn new() -> SceneMetrics {
        Self {
            sma_dt: SimpleMovingAverage::new(100),
            sma_render_loop: SimpleMovingAverage::new(100),
            sma_render_time: SimpleMovingAverage::new(100),
            sma_swap_time: SimpleMovingAverage::new(100),
            sma_tick_time: SimpleMovingAverage::new(100),
        }
    }

    pub fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("Metrics")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([200.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Avg FPS: {:.1}", 1.0 / self.sma_dt.get()));
                // Time physics simulation of the scene took
                ui.text(format!(
                    "Scene: time to tick: {:.1} micro-s",
                    self.sma_tick_time.get()
                ));
                // Time it took to pass rendering logic and GPU command buffers
                ui.text(format!(
                    "Scene: time to render: {:.1} micro-s",
                    self.sma_render_time.get(),
                ));
                // Time it took to swap buffers. This is somehow representative of time
                // that was spent waiting for the GPU (incl. any delay for VSync)
                ui.text(format!(
                    "Swap time: {:.1} micro-s",
                    self.sma_swap_time.get()
                ));
                ui.text(format!(
                    "Avg time per render loop: {:.1} micro-s",
                    self.sma_render_loop.get()
                ));
            });
    }
}
