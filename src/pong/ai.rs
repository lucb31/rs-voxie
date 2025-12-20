use glam::{Vec3, Vec4Swizzles};
use hecs::{Entity, World};

use crate::systems::physics::Transform;

use super::{
    ball::PongBall,
    paddle::{PongPaddle, spawn_paddle},
};

pub struct PongAi;

pub fn spawn_ai(world: &mut World, position: Vec3) {
    let paddle = spawn_paddle(world, position);
    world
        .insert(paddle, (PongAi,))
        .expect("Could not add ai. Missing paddle entity");
}

fn get_ball_position(world: &mut World) -> Option<Vec3> {
    let mut query = world.query::<&Transform>().with::<&PongBall>();
    let (_entity, transform) = query.iter().next()?;
    Some(transform.0.w_axis.xyz())
}

/// AI logic to set input velocity of controlled paddle
pub fn system_ai(world: &mut World) {
    let ball_position = get_ball_position(world);
    let paddle_query = world
        .query_mut::<(&Transform, &mut PongPaddle)>()
        .with::<&PongAi>();
    for (_entity, (transform, paddle)) in paddle_query {
        if let Some(ball) = ball_position {
            let delta_y = ball.y - transform.0.w_axis.y;
            if delta_y.abs() > 0.2 {
                paddle.input_velocity = delta_y.signum() * Vec3::Y;
            } else {
                paddle.input_velocity = Vec3::ZERO;
            }
        } else {
            paddle.input_velocity = Vec3::ZERO;
        }
    }
}
