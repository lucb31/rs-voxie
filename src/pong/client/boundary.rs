use glam::{Mat4, Quat, Vec3};
use hecs::World;

use crate::{
    renderer::{
        RenderMeshHandle,
        ecs_renderer::{MESH_CUBE, RenderColor},
    },
    systems::physics::Transform,
    util::despawn_all,
};

use crate::collision::ColliderBody;

pub(super) struct PongBallTrigger {
    pub(super) player_id: usize,
}
struct PongBoundary;

pub fn spawn_boundaries(world: &mut World, width: f32, height: f32) {
    let thicknes = 0.25;
    let render_mesh_handle = RenderMeshHandle(MESH_CUBE);
    let horizontal_scale = Vec3::new(width - thicknes * 1.01, thicknes, 1.0);
    let vertical_scale = Vec3::new(thicknes, height, 1.0);
    world.spawn_batch([
        (
            // top
            Transform(Mat4::from_scale_rotation_translation(
                horizontal_scale,
                Quat::IDENTITY,
                Vec3::new(0.0, -height / 2.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
            ColliderBody::AabbCollider {
                scale: horizontal_scale,
            },
            PongBoundary,
        ),
        (
            // bottom
            Transform(Mat4::from_scale_rotation_translation(
                horizontal_scale,
                Quat::IDENTITY,
                Vec3::new(0.0, height / 2.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
            ColliderBody::AabbCollider {
                scale: horizontal_scale,
            },
            PongBoundary,
        ),
    ]);
    world.spawn_batch([
        (
            // left
            Transform(Mat4::from_scale_rotation_translation(
                vertical_scale,
                Quat::IDENTITY,
                Vec3::new(-width / 2.0, 0.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
            ColliderBody::AabbCollider {
                scale: vertical_scale,
            },
            PongBallTrigger { player_id: 0 },
            PongBoundary,
        ),
        (
            // right
            Transform(Mat4::from_scale_rotation_translation(
                vertical_scale,
                Quat::IDENTITY,
                Vec3::new(width / 2.0, 0.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
            ColliderBody::AabbCollider {
                scale: vertical_scale,
            },
            PongBallTrigger { player_id: 1 },
            PongBoundary,
        ),
    ]);
}

pub fn despawn_boundaries(world: &mut World) {
    despawn_all::<&PongBoundary>(world);
}
