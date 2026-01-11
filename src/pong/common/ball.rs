use glam::{Mat4, Quat, Vec3};
use hecs::{Entity, World};
use log::info;

use super::{boundary::PongBallTrigger, paddle::PaddleControl};

use crate::{
    collision::CollisionEvent,
    network::{Authority, NetEntityId, NetworkReplicated, NetworkWorld},
    systems::physics::{Transform, Velocity},
};

use crate::collision::ColliderBody;

pub const BALL_MIN_SPEED: f32 = 1.0;
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
    let speed = BALL_MIN_SPEED;
    let (net_id, entity) = world.spawn(
        (
            PongBall { speed, bounces: 0 },
            Transform(Mat4::from_scale_rotation_translation(
                scale,
                Quat::IDENTITY,
                Vec3::ZERO,
            )),
            ColliderBody::SphereCollider { radius: 0.125 },
            NetworkReplicated {
                authority: Authority::Server,
            },
        ),
        net_entity_id,
    );
    #[cfg(feature = "gui")]
    {
        world
            .get_world_mut()
            .insert(
                entity,
                (
                    crate::renderer::RenderMeshHandle(
                        crate::renderer::ecs_renderer::MESH_PROJECTILE_2D,
                    ),
                    crate::renderer::ecs_renderer::RenderColor(Vec3::ONE),
                ),
            )
            .expect("Could not add rendering info to ball.");
    }
    (net_id, entity)
}

/// Returns slot_number of **loosing** player, if game over
pub fn bounce_balls(world: &mut World, collisions: &Vec<CollisionEvent>) -> Option<usize> {
    if collisions.is_empty() {
        return None;
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
                info!("Game over. Player {:?} lost", trigger.player_slot);
                ball.speed = 0.0;
                velocity.0 = Vec3::ZERO;
                return Some(trigger.player_slot);
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
            if world.get::<&PaddleControl>(other).is_ok() {
                ball.bounces += 1;
                ball.speed = exp_lerp(
                    BALL_MIN_SPEED,
                    MAX_SPEED,
                    ball.bounces as f32 / MAX_BOUNCES as f32,
                );
                info!("Bounce #{}: New speed = {}", ball.bounces, ball.speed);
            }
        }
    }
    None
}

fn exp_lerp(min_val: f32, max_val: f32, t: f32) -> f32 {
    min_val * (max_val / min_val).powf(t)
}
