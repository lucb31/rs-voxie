use glam::{Vec3, Vec4Swizzles};
use hecs::{Entity, World};

use crate::{
    network::{Authority, NetEntityId, NetworkReplicated, NetworkWorld},
    pong::common::{
        ball::PongBall,
        paddle::{PaddleControl, PaddleSpeed, spawn_paddle},
    },
    systems::physics::Transform,
    util::smooth_damp,
};

pub struct PongAi {
    velocity_smooth: Vec3,
}

pub fn spawn_ai(
    world: &mut NetworkWorld,
    net_entity_id: Option<NetEntityId>,
) -> (NetEntityId, Entity) {
    let (net_entity_id, paddle) = spawn_paddle(world, 1, net_entity_id);
    world
        .get_world_mut()
        .insert(
            paddle,
            (
                PongAi {
                    velocity_smooth: Vec3::ZERO,
                },
                NetworkReplicated {
                    authority: Authority::Server,
                },
            ),
        )
        .expect("Could not add ai. Missing paddle entity");
    world
        .get_world_mut()
        .exchange_one::<PaddleSpeed, PaddleSpeed>(paddle, PaddleSpeed { speed: 4.0 })
        .expect("Could not update paddle speed");
    (net_entity_id, paddle)
}

fn get_ball_position(world: &mut World) -> Option<Vec3> {
    let mut query = world.query::<&Transform>().with::<&PongBall>();
    let (_entity, transform) = query.iter().next()?;
    Some(transform.0.w_axis.xyz())
}

/// AI logic to set input velocity of controlled paddle
pub fn system_ai(world: &mut World, dt: f32) {
    let ball_position = get_ball_position(world);
    let paddle_query =
        world.query_mut::<(&Transform, &PaddleSpeed, &mut PaddleControl, &mut PongAi)>();
    for (_entity, (transform, speed, paddle, ai)) in paddle_query {
        if let Some(ball) = ball_position {
            let target_velocity = Vec3::new(
                0.0,
                ((ball.y - transform.0.w_axis.y) / dt).clamp(-speed.speed, speed.speed),
                0.0,
            );
            paddle.input_velocity = smooth_damp(
                paddle.input_velocity,
                target_velocity,
                &mut ai.velocity_smooth,
                0.01,
                dt,
            );
        } else {
            paddle.input_velocity = Vec3::ZERO;
        }
    }
}
