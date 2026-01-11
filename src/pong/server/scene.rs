use std::{error::Error, time::Instant};

use glow::HasContext;
use log::info;

use crate::{
    collision::{CollisionEvent, system_collisions},
    config::BROADCAST_DT,
    log_err,
    network::NetworkWorld,
    pong::{
        BincodeCodec, ServerProtocol,
        common::{
            ball::{PongBall, bounce_balls},
            paddle::{PaddleControl, system_paddle_movement},
            setup_static_entities,
        },
        network::ServerMessage,
    },
    scenes::scene::BaseScene,
    systems::physics::system_movement,
};

use super::{
    lobby::Lobby,
    player::apply_player_inputs,
    sync::{server_process_client_message, server_send_snapshots},
};

pub(super) enum ServerGameState {
    WaitingForPlayers,
    Running,
}

pub struct PongServerScene {
    collisions: Vec<CollisionEvent>,
    game_state: ServerGameState,
    world: NetworkWorld,
    protocol: ServerProtocol<BincodeCodec>,
    lobby: Lobby,
    server_tick: u32,

    last_broadcast: Instant,
}

impl PongServerScene {
    pub fn new(protocol: ServerProtocol<BincodeCodec>) -> Result<PongServerScene, Box<dyn Error>> {
        let mut world = NetworkWorld::new();
        setup_static_entities(&mut world);
        Ok(Self {
            protocol,
            collisions: Vec::new(),
            game_state: ServerGameState::WaitingForPlayers,
            world,
            lobby: Lobby::new(),
            server_tick: 0,
            last_broadcast: Instant::now(),
        })
    }

    fn end_round(&mut self, looser_slot: usize) {
        info!(
            "[T{}] Ending round. Player {} lost",
            self.server_tick, looser_slot
        );

        // Broadcast game over
        self.protocol
            .broadcast(ServerMessage::EndRound {
                server_tick: self.server_tick,
                loosing_player_slot: looser_slot,
            })
            .expect("Failed to broadcast end of round");
        // Despawn on server
        log_err!(
            self.world.despawn_all::<&PongBall>(),
            "Could not despawn balls {err}"
        );
        log_err!(
            self.world.despawn_all::<&PaddleControl>(),
            "Could not despawn paddles {err}"
        );
        self.game_state = ServerGameState::WaitingForPlayers;
        // Reset lobby & frame
        self.lobby = Lobby::new();
        self.server_tick = 0;
    }

    fn tick(&mut self, dt: f32) {
        while let Some(message) = self.protocol.try_recv() {
            server_process_client_message(
                &mut self.world,
                message,
                &self.protocol,
                &mut self.game_state,
                &mut self.lobby,
                self.server_tick,
            );
        }
        if matches!(self.game_state, ServerGameState::Running) {
            apply_player_inputs(&mut self.world, &mut self.lobby);
            // Collision systems
            self.collisions = system_collisions(self.world.get_world_mut());
            let loosing_player = bounce_balls(self.world.get_world_mut(), &self.collisions);
            if let Some(loosing_player_slot) = loosing_player {
                self.end_round(loosing_player_slot);
            }
            system_paddle_movement(self.world.get_world_mut(), &self.collisions);

            // Physics simulation
            system_movement(self.world.get_world_mut(), dt);

            // Broadcast
            if self.last_broadcast.elapsed() >= BROADCAST_DT {
                server_send_snapshots(&self.world, &self.protocol, &self.lobby, self.server_tick);
                self.last_broadcast = Instant::now();
            }
        }

        self.server_tick += 1;
    }
}

impl BaseScene for PongServerScene {
    fn get_world(&self) -> Option<&hecs::World> {
        Some(self.world.get_world())
    }

    fn get_title(&self) -> String {
        "Pong server".to_string()
    }

    fn tick(&mut self, dt: f32) {
        PongServerScene::tick(self, dt);
    }

    fn start(&mut self) {}
}

#[cfg(feature = "gui")]
impl crate::scenes::scene::GuiScene for PongServerScene {
    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("Scene info")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([500.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Tick: {}", self.server_tick));
            });
    }
}
