use std::error::Error;

use log::info;

use crate::{
    collision::{CollisionEvent, system_collisions},
    log_err,
    network::{NetworkWorld, ServerScene},
    pong::{
        JsonCodec, ServerProtocol,
        client::{
            ai::system_ai,
            ball::{PongBall, bounce_balls},
            boundary::spawn_boundaries,
            paddle::{PongPaddle, system_paddle_movement},
        },
        network::ServerMessage,
    },
    systems::physics::system_movement,
};

use super::sync::{server_broadcast_transform_state, server_process_client_message};

pub struct PongServerScene {
    collisions: Vec<CollisionEvent>,
    game_over: bool,
    world: NetworkWorld,
    protocol: ServerProtocol<JsonCodec>,
}

impl PongServerScene {
    pub fn new(protocol: ServerProtocol<JsonCodec>) -> Result<PongServerScene, Box<dyn Error>> {
        let mut world = NetworkWorld::new();
        let width = 5.0;
        let height = 5.0;
        spawn_boundaries(world.get_world_mut(), width, height);
        Ok(Self {
            protocol,
            collisions: Vec::new(),
            game_over: true,
            world,
        })
    }

    fn end_round(&mut self) {
        info!("Ending round");
        // Broadcast game over
        self.protocol
            .broadcast(ServerMessage::ServerEndRound { winner: 99 })
            .expect("Failed to broadcast end of round");
        // Despawn on server
        log_err!(
            self.world.despawn_all::<&PongBall>(),
            "Could not despawn balls {err}"
        );
        log_err!(
            self.world.despawn_all::<&PongPaddle>(),
            "Could not despawn paddles {err}"
        );
        self.game_over = true;
    }

    fn tick(&mut self, dt: f32) {
        while let Some(cmd) = self.protocol.try_recv() {
            server_process_client_message(
                &mut self.world,
                cmd,
                &self.protocol,
                &mut self.game_over,
            );
        }
        system_ai(self.world.get_world_mut(), dt);

        // Collision systems
        self.collisions = system_collisions(self.world.get_world_mut());
        let game_over = bounce_balls(self.world.get_world_mut(), &self.collisions);
        if game_over {
            self.end_round();
        }
        system_paddle_movement(self.world.get_world_mut(), &self.collisions);

        // Physics simulation
        system_movement(self.world.get_world_mut(), dt);
    }
}
impl ServerScene for PongServerScene {
    fn tick(&mut self, dt: f32) {
        PongServerScene::tick(self, dt);
    }

    fn get_title(&self) -> String {
        "Pong server".to_string()
    }

    fn broadcast_state(&self) {
        if !self.game_over {
            server_broadcast_transform_state(&self.world, &self.protocol);
        }
    }
}
