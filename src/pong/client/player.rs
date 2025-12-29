use glam::Vec3;
use hecs::{Entity, World};
use winit::keyboard::KeyCode;

use crate::{
    input::InputState,
    network::{NetEntityId, NetworkWorld},
};

use super::paddle::{PongPaddle, spawn_paddle};

pub struct PongPlayer;

pub fn spawn_player(
    world: &mut NetworkWorld,
    position: Vec3,
    net_entity_id: Option<NetEntityId>,
) -> (NetEntityId, Entity) {
    let (net_id, paddle) = spawn_paddle(world, position, net_entity_id);
    world
        .get_world_mut()
        .insert(paddle, (PongPlayer,))
        .expect("Could not add player. Missing paddle entity");
    (net_id, paddle)
}

/// Parse keyboard inputs to set paddle input velocity
pub fn system_player_input(world: &mut World, input: &InputState) {
    for (_entity, paddle) in world.query_mut::<&mut PongPaddle>().with::<&PongPlayer>() {
        // Parse inputs
        let mut input_velocity = Vec3::ZERO;
        if input.is_key_pressed(&KeyCode::KeyW) {
            input_velocity += Vec3::Y;
        }
        if input.is_key_pressed(&KeyCode::KeyS) {
            input_velocity -= Vec3::Y;
        }
        // Directly to max speed.
        // Improvement: Smoothing / acceleration
        paddle.input_velocity = input_velocity * paddle.speed;
    }
}
