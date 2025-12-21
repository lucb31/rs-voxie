use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use glow::HasContext;
use hecs::{Entity, World};

use crate::{
    collision::{ColliderBody, CollisionEvent},
    meshes::objmesh::ObjMesh,
    renderer::{Mesh, RenderMeshHandle, ecs_renderer::MESH_CUBE, shader::Shader},
    systems::physics::{Transform, Velocity},
    util::despawn_all,
};
pub struct PongPaddle {
    pub(super) speed: f32,
    pub(super) input_velocity: Vec3,
}

pub fn spawn_paddle(world: &mut World, position: Vec3) -> Entity {
    let scale = Vec3::new(0.1, 1.0, 1.0);
    world.spawn((
        Transform(Mat4::from_scale_rotation_translation(
            scale,
            Quat::IDENTITY,
            position,
        )),
        Velocity(Vec3::ZERO),
        RenderMeshHandle(MESH_CUBE),
        PongPaddle {
            speed: 2.0,
            input_velocity: Vec3::ZERO,
        },
        ColliderBody::AabbCollider { scale },
    ))
}

pub fn despawn_paddles(world: &mut World) {
    despawn_all::<&PongPaddle>(world);
}

/// Calculate paddle velocity based on requested velocity and collide_and_slide algorithm
/// Integration of velocity is done in general movement system
pub fn system_paddle_movement(world: &mut World, collisions: &[CollisionEvent]) {
    for (entity, (transform, velocity, movement)) in
        world.query_mut::<(&Transform, &mut Velocity, &PongPaddle)>()
    {
        let mut input_velocity = movement.input_velocity;
        debug_assert!(
            input_velocity.length_squared() <= movement.speed * movement.speed,
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
