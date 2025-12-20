use glam::{Vec3, Vec4Swizzles};
use hecs::World;

use crate::{systems::physics::Transform, util::smooth_damp};

use super::{
    ball::PongBall,
    paddle::{PongPaddle, spawn_paddle},
};

pub struct PongAi {
    velocity_smooth: Vec3,
}

pub fn spawn_ai(world: &mut World, position: Vec3) {
    let paddle = spawn_paddle(world, position);
    world
        .insert(
            paddle,
            (PongAi {
                velocity_smooth: Vec3::ZERO,
            },),
        )
        .expect("Could not add ai. Missing paddle entity");
}

fn get_ball_position(world: &mut World) -> Option<Vec3> {
    let mut query = world.query::<&Transform>().with::<&PongBall>();
    let (_entity, transform) = query.iter().next()?;
    Some(transform.0.w_axis.xyz())
}

/// AI logic to set input velocity of controlled paddle
pub fn system_ai(world: &mut World, dt: f32) {
    let ball_position = get_ball_position(world);
    let paddle_query = world.query_mut::<(&Transform, &mut PongPaddle, &mut PongAi)>();
    for (_entity, (transform, paddle, ai)) in paddle_query {
        if let Some(ball) = ball_position {
            let target_velocity = Vec3::new(
                0.0,
                (ball.y - transform.0.w_axis.y).clamp(-paddle.speed, paddle.speed),
                0.0,
            );
            paddle.input_velocity = smooth_damp(
                paddle.input_velocity,
                target_velocity,
                &mut ai.velocity_smooth,
                0.1,
                dt,
            );
        } else {
            paddle.input_velocity = Vec3::ZERO;
        }
    }
}
