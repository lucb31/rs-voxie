use std::{
    thread,
    time::{Duration, Instant},
};

use log::info;

use crate::{
    application::{BROADCAST_DT, SIMULATION_DT},
    scenes::Scene,
};

/// Runs simulation of scene without rendering
pub struct HeadlessSimulation {
    scene: Box<dyn Scene>,
}

impl HeadlessSimulation {
    pub fn new(scene: Box<dyn Scene>) -> Self {
        Self { scene }
    }

    /// Sleep for broadcast tick duration, then simulate multiple ticks
    /// in one go
    pub fn run(&mut self) {
        info!("Starting headless simulation: {}", self.scene.get_title());
        let mut last_instant = Instant::now();
        let tick_duration = SIMULATION_DT;
        let broadcast_duration = BROADCAST_DT;

        let mut tick_accumulator = Duration::ZERO;

        loop {
            let now = Instant::now();
            let delta = now - last_instant;
            last_instant = now;

            tick_accumulator += delta;

            // Run simulation ticks for every tick_duration that has passed
            while tick_accumulator >= tick_duration {
                self.scene.tick(tick_duration.as_secs_f32());
                tick_accumulator -= tick_duration;
            }

            // Sleep until next broadcast to avoid busy waiting
            thread::sleep(broadcast_duration);
        }
    }
}
