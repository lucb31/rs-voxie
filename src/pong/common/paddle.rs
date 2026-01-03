use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use hecs::{Entity, World};

use crate::{
    collision::{ColliderBody, CollisionEvent},
    network::{Authority, NetEntityId, NetworkReplicated, NetworkWorld},
    renderer::{
        RenderMeshHandle,
        ecs_renderer::{MESH_CUBE, RenderColor},
    },
    systems::physics::{Transform, Velocity},
};
pub(crate) struct PaddleId {
    pub(crate) slot: usize,
}
pub(crate) struct PaddleSpeed {
    pub(crate) speed: f32,
}
pub struct PaddleControl {
    pub(crate) input_velocity: Vec3,
}
const PLAYER_SPAWN_POSITIONS: [Vec3; 2] = [Vec3::new(-2.3, 0.0, 0.0), Vec3::new(2.3, 0.0, 0.0)];

pub fn spawn_paddle(
    world: &mut NetworkWorld,
    player_slot: usize,
    net_entity_id: Option<NetEntityId>,
) -> (NetEntityId, Entity) {
    let scale = Vec3::new(0.1, 1.0, 1.0);
    world.spawn(
        (
            Transform(Mat4::from_scale_rotation_translation(
                scale,
                Quat::IDENTITY,
                PLAYER_SPAWN_POSITIONS[player_slot],
            )),
            Velocity(Vec3::ZERO),
            RenderMeshHandle(MESH_CUBE),
            RenderColor(Vec3::X),
            PaddleId { slot: player_slot },
            PaddleSpeed { speed: 2.0 },
            PaddleControl {
                input_velocity: Vec3::ZERO,
            },
            NetworkReplicated {
                authority: Authority::Server,
            },
            ColliderBody::AabbCollider { scale },
        ),
        net_entity_id,
    )
}

/// Calculate paddle velocity based on requested velocity and collide_and_slide algorithm
/// Integration of velocity is done in general movement system
pub fn system_paddle_movement(world: &mut World, collisions: &[CollisionEvent]) {
    for (entity, (transform, velocity, movement, speed)) in
        world.query_mut::<(&Transform, &mut Velocity, &PaddleControl, &PaddleSpeed)>()
    {
        let mut input_velocity = movement.input_velocity;
        debug_assert!(
            input_velocity.length_squared() <= speed.speed * speed.speed,
            "Too high input velocity requested {input_velocity}",
        );
        if input_velocity.length_squared() < 1e-4 {
            velocity.0 = Vec3::ZERO;
        } else {
            // Restrict vertical movement when colliding with top or bottom boundary
            let relevant_collisions = collisions
                .iter()
                .filter(|e| e.a == entity || e.b == Some(entity));
            let current_position = transform.0.w_axis.xyz();
            for collision in relevant_collisions {
                if collision.info.contact_point.y > current_position.y {
                    input_velocity.y = input_velocity.y.min(0.0);
                } else {
                    input_velocity.y = input_velocity.y.max(0.0);
                }
            }

            velocity.0 = input_velocity;
        }
    }
}
