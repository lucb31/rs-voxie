use glam::Vec3;
use hecs::World;
use winit::keyboard::KeyCode;

use crate::input::InputState;

use super::paddle::{PongPaddle, spawn_paddle};

pub struct PongPlayer;

pub fn spawn_player(world: &mut World, position: Vec3) {
    let paddle = spawn_paddle(world, position);
    world
        .insert(paddle, (PongPlayer,))
        .expect("Could not add player. Missing paddle entity");
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
