use crate::util::SimpleMovingAverage;

pub struct ApplicationMetrics {
    pub sma_dt: SimpleMovingAverage,
    pub sma_render_loop: SimpleMovingAverage,
    pub sma_render_time: SimpleMovingAverage,
    pub sma_swap_time: SimpleMovingAverage,
    pub sma_tick_time: SimpleMovingAverage,
}

impl ApplicationMetrics {
    pub fn new() -> ApplicationMetrics {
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
