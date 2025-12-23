use glam::{Mat4, Quat, Vec3};
use hecs::{Entity, World};
use log::info;

use super::{boundary::PongBallTrigger, paddle::PongPaddle};

use crate::{
    collision::CollisionEvent,
    network::{NetEntityId, NetworkWorld},
    renderer::{RenderMeshHandle, ecs_renderer::MESH_PROJECTILE_2D},
    systems::physics::{Transform, Velocity},
    util::despawn_all,
};

use crate::collision::ColliderBody;

pub const MIN_SPEED: f32 = 1.0;
const MAX_SPEED: f32 = 4.5;
// Number of paddle bounces until max_speed will be reached
const MAX_BOUNCES: usize = 25;

pub struct PongBall {
    pub speed: f32,
    pub bounces: usize,
}

pub fn spawn_ball(
    world: &mut NetworkWorld,
    net_entity_id: Option<NetEntityId>,
) -> (NetEntityId, Entity) {
    let scale = Vec3::splat(0.25);
    let direction = Vec3::new(1.0, 0.5, 0.0).normalize();
    let speed = MIN_SPEED;
    world.spawn(
        (
            PongBall { speed, bounces: 0 },
            Transform(Mat4::from_scale_rotation_translation(
                scale,
                Quat::IDENTITY,
                Vec3::ZERO,
            )),
            Velocity(direction * speed),
            RenderMeshHandle(MESH_PROJECTILE_2D),
            ColliderBody::SphereCollider { radius: 0.125 },
        ),
        net_entity_id,
    )
}

pub fn despawn_balls(world: &mut World) {
    despawn_all::<&PongBall>(world);
}

/// Returns true if game over
pub fn bounce_balls(world: &mut World, collisions: &Vec<CollisionEvent>) -> bool {
    if collisions.is_empty() {
        return false;
    }
    let mut ball_query = world.query::<(&mut Transform, &mut Velocity, &mut PongBall)>();
    for (ball_entity, (ball_transform, velocity, ball)) in ball_query.iter() {
        for collision in collisions {
            if collision.a != ball_entity && collision.b != Some(ball_entity) {
                // Skip collisions where ball is not involved
                continue;
            }
            let other = if collision.a == ball_entity {
                collision.b.unwrap()
            } else {
                collision.a
            };
            // Game over if we've hit a trigger
            if let Ok(trigger) = world.get::<&PongBallTrigger>(other) {
                info!("Game over. Player {} lost", trigger.player_id);
                ball.speed = 0.0;
                velocity.0 = Vec3::ZERO;
                return true;
            }

            let info = collision.info;
            debug_assert!(info.normal.is_finite(), "Received infinite normal");
            // Resolve penetration
            let d_penetration = info.normal * info.penetration_depth;
            ball_transform.0.w_axis.x += d_penetration.x;
            ball_transform.0.w_axis.y += d_penetration.y;
            ball_transform.0.w_axis.z += d_penetration.z;

            // Reflect velocity
            let reflected_velocity = velocity.0 - 2.0 * velocity.0.dot(info.normal) * info.normal;
            // Alternative A: Scale to speed
            //velocity.0 = reflected_velocity.normalize() * ball.speed;
            // Alternative B: Fixed x-speed
            let x_multiplier = (ball.speed / reflected_velocity.x).abs();
            velocity.0 = reflected_velocity * x_multiplier;

            // Increase speed if we've hit a paddle
            if world.get::<&PongPaddle>(other).is_ok() {
                ball.bounces += 1;
                ball.speed = exp_lerp(
                    MIN_SPEED,
                    MAX_SPEED,
                    ball.bounces as f32 / MAX_BOUNCES as f32,
                );
                info!("Bounce #{}: New speed = {}", ball.bounces, ball.speed);
            }
        }
    }
    false
}

fn exp_lerp(min_val: f32, max_val: f32, t: f32) -> f32 {
    min_val * (max_val / min_val).powf(t)
}
