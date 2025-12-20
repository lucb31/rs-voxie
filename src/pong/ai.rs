use glam::Vec3;
use hecs::World;

use super::paddle::{PongPaddle, spawn_paddle};

pub struct PongAi;

pub fn spawn_ai(world: &mut World, position: Vec3) {
    let paddle = spawn_paddle(world, position);
    world
        .insert(paddle, (PongAi,))
        .expect("Could not add ai. Missing paddle entity");
}

/// AI logic to set input velocity of controlled paddle
pub fn system_ai(world: &mut World) {
    for (_entity, paddle) in world.query_mut::<&mut PongPaddle>().with::<&PongAi>() {
        paddle.input_velocity = Vec3::Y;
    }
}
