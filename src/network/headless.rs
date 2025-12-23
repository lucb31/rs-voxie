use std::{
    sync::mpsc::Sender,
    thread,
    time::{Duration, Instant},
};

use log::{info, warn};

use crate::{log_err, systems::physics::Transform};

use super::{NetEntityId, NetworkCommand, NetworkScene};

/// Runs simulation of scene without rendering
pub(super) struct HeadlessSimulation {
    scene: Box<dyn NetworkScene>,
    broadcast_channel: Sender<NetworkCommand>,
    broadcasts_per_second: u64,
    ticks_per_second: u64,
}

impl HeadlessSimulation {
    pub fn new(scene: Box<dyn NetworkScene>, broadcast_channel: Sender<NetworkCommand>) -> Self {
        Self {
            scene,
            broadcast_channel,
            broadcasts_per_second: 5,
            ticks_per_second: 60,
        }
    }

    /// Sleep for broadcast tick duration, then simulate multiple ticks
    /// in one go
    pub fn run(&mut self) {
        info!("Starting headless simulation: {}", self.scene.get_title());
        self.scene.start_match();
        warn!("Hard-coded ball spawn");
        self.broadcast_channel
            .send(NetworkCommand::ServerSpawnBall { net_entity_id: 0 })
            .unwrap();

        let mut last_instant = Instant::now();
        let tick_duration = Duration::from_nanos(1_000_000_000 / self.ticks_per_second);
        let broadcast_duration = Duration::from_nanos(1_000_000_000 / self.broadcasts_per_second);

        let mut tick_accumulator = Duration::ZERO;
        let mut broadcast_accumulator = Duration::ZERO;

        loop {
            let now = Instant::now();
            let delta = now - last_instant;
            last_instant = now;

            tick_accumulator += delta;
            broadcast_accumulator += delta;

            // Run simulation ticks for every tick_duration that has passed
            while tick_accumulator >= tick_duration {
                self.scene.tick(tick_duration.as_secs_f32());
                tick_accumulator -= tick_duration;
                if self.scene.game_over() {
                    warn!("Match over. Exiting game loop");
                    return;
                }
            }

            // Broadcast if enough time has passed
            if broadcast_accumulator >= broadcast_duration {
                self.broadcast_transform_state();
                broadcast_accumulator -= broadcast_duration;
            }

            // Sleep until next broadcast to avoid busy waiting
            let sleep_duration = broadcast_duration
                .checked_sub(broadcast_accumulator)
                .unwrap_or(Duration::ZERO);
            thread::sleep(sleep_duration);
        }
    }

    fn broadcast_transform_state(&mut self) {
        let world = self.scene.get_world();
        let channel = self.broadcast_channel.clone();
        for (_entity, transform) in world.query::<&Transform>().iter() {
            warn!("Hard-coded ball net entity id");
            let net_entity_id: NetEntityId = 0;
            let cmd: NetworkCommand = NetworkCommand::ServerUpdateTransform {
                net_entity_id,
                transform: transform.clone(),
            };
            log_err!(channel.send(cmd), "Failure broadcasting command: {err}");
        }
    }
}
