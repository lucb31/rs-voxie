use std::{error::Error, time::Instant};

use glow::HasContext;
use log::info;

use crate::{
    application::BROADCAST_DT,
    collision::{CollisionEvent, system_collisions},
    log_err,
    network::NetworkWorld,
    pong::{
        BincodeCodec, ServerProtocol,
        client::ai::system_ai,
        common::{
            ball::{PongBall, bounce_balls},
            paddle::{PaddleControl, system_paddle_movement},
            setup_static_entities,
        },
        network::ServerMessage,
    },
    scenes::Scene,
    systems::physics::system_movement,
};

use super::{
    lobby::Lobby,
    sync::{server_broadcast_transform_state, server_process_client_message},
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
    frame: u32,

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
            frame: 0,
            last_broadcast: Instant::now(),
        })
    }

    fn end_round(&mut self, looser_slot: usize) {
        info!("Ending round");

        // Broadcast game over
        self.protocol
            .broadcast(ServerMessage::EndRound {
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
        self.frame = 0;
    }

    fn tick(&mut self, dt: f32) {
        while let Some(message) = self.protocol.try_recv() {
            server_process_client_message(
                &mut self.world,
                message,
                &self.protocol,
                &mut self.game_state,
                &mut self.lobby,
                self.frame,
            );
        }
        system_ai(self.world.get_world_mut(), dt);

        // Collision systems
        self.collisions = system_collisions(self.world.get_world_mut());
        let loosing_player = bounce_balls(self.world.get_world_mut(), &self.collisions);
        if let Some(loosing_player_slot) = loosing_player {
            self.end_round(loosing_player_slot);
        }
        system_paddle_movement(self.world.get_world_mut(), &self.collisions);

        // Physics simulation
        system_movement(self.world.get_world_mut(), dt);
        self.frame += 1;

        // Broadcast
        if self.last_broadcast.elapsed() >= BROADCAST_DT {
            self.broadcast_state();
            self.last_broadcast = Instant::now();
        }
    }

    fn broadcast_state(&self) {
        if matches!(self.game_state, ServerGameState::Running) {
            server_broadcast_transform_state(&self.world, &self.protocol, self.frame);
        }
    }
}

impl Scene for PongServerScene {
    fn get_world(&self) -> Option<&hecs::World> {
        Some(self.world.get_world())
    }

    fn get_title(&self) -> String {
        "Pong server".to_string()
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn tick(&mut self, dt: f32) {
        PongServerScene::tick(self, dt);
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn render_ui(&mut self, ui: &mut imgui::Ui) {}

    fn start(&mut self) {}
}
