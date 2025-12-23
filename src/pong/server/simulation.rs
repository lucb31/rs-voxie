use std::{
    sync::mpsc::Sender,
    thread,
    time::{Duration, Instant},
};

use glam::Vec3;
use hecs::World;
use log::{error, info, trace};

use crate::{
    collision::{CollisionEvent, system_collisions},
    log_err,
    network::{NetEntityId, NetworkCommand},
    pong::client::{
        ai::spawn_ai,
        ball::{PongBall, bounce_balls, spawn_ball},
        boundary::spawn_boundaries,
        paddle::system_paddle_movement,
    },
    systems::physics::{Transform, system_movement},
};

const TICK_RATE: u64 = 60;
const TICK_DURATION: Duration = Duration::from_nanos(1_000_000_000 / TICK_RATE);

/// One game instance
pub(super) struct PongSimulation {
    collisions: Vec<CollisionEvent>,
    game_over: bool,

    world: World,
    broadcast_tx: Option<Sender<NetworkCommand>>,
}

impl PongSimulation {
    pub fn new() -> PongSimulation {
        Self {
            world: World::new(),
            collisions: Vec::new(),
            game_over: true,
            broadcast_tx: None,
        }
    }

    pub fn run(&mut self, tx: Sender<NetworkCommand>) {
        self.broadcast_tx = Some(tx);
        self.start_round();
        let mut next_tick = Instant::now();
        let mut last_tick = Instant::now();
        let mut frame: usize = 0;

        loop {
            if self.game_over {
                // Break game loop if game over
                return;
            }
            self.tick(last_tick.elapsed().as_secs_f32());
            frame += 1;
            if frame % 5 == 0 {
                // Broadcast updates at reduced rate
                self.broadcast_updates();
            }

            next_tick += TICK_DURATION;
            let now = Instant::now();
            last_tick = now;

            if next_tick > now {
                thread::sleep(next_tick - now);
            } else {
                // tick behind schedule – skip sleep
            }
        }
    }

    fn broadcast_updates(&mut self) {
        // Ball transform updates
        if let Some((_entity, (ball_transform, _))) =
            self.world.query::<(&Transform, &PongBall)>().iter().next()
        {
            // TODO: Hard-coded until we have server side net id map
            let ball_net_entity_id: NetEntityId = 0;
            let cmd: NetworkCommand = NetworkCommand::UpdateTransform {
                net_entity_id: ball_net_entity_id,
                transform: ball_transform.clone(),
            };
            self.send_cmd_unreliable(cmd);
        }
    }

    fn send_cmd_unreliable(&self, cmd: NetworkCommand) {
        match &self.broadcast_tx {
            Some(tx) => {
                log_err!(tx.send(cmd), "Failure broadcasting command: {err}");
            }
            None => error!("Cannot send command: Missing command broadcast channel."),
        }
    }

    fn start_round(&mut self) {
        info!("Starting round");

        spawn_ai(&mut self.world, Vec3::new(2.3, 0.0, 0.0));
        spawn_ball(&mut self.world);
        let spawn_cmd = NetworkCommand::SpawnBall { net_entity_id: 0 };
        self.send_cmd_unreliable(spawn_cmd);
        let width = 5.0;
        let height = 5.0;
        spawn_boundaries(&mut self.world, width, height);
        self.game_over = false;
    }

    fn end_round(&mut self) {
        info!("Ending round");
        self.game_over = true;
    }

    fn tick(&mut self, dt: f32) {
        trace!("Tick {dt}");
        // Collision systems
        self.collisions = system_collisions(&mut self.world);

        trace!("Collisions {}", self.collisions.len());

        let game_over = bounce_balls(&mut self.world, &self.collisions);
        if game_over {
            self.end_round();
        }
        system_paddle_movement(&mut self.world, &self.collisions);

        // Physics simulation
        system_movement(&mut self.world, dt);
    }
}
