use glam::{Mat4, Quat, Vec3};
use hecs::World;

use crate::{
    renderer::{
        RenderMeshHandle,
        ecs_renderer::{MESH_CUBE, RenderColor},
    },
    systems::physics::Transform,
};

pub fn spawn_boundaries(world: &mut World, width: f32, height: f32) {
    let thicknes = 0.25;
    let render_mesh_handle = RenderMeshHandle(MESH_CUBE);
    world.spawn_batch([
        (
            // top
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::new(width, thicknes, 1.0),
                Quat::IDENTITY,
                Vec3::new(0.0, -height / 2.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
        ),
        (
            // bottom
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::new(width, thicknes, 1.0),
                Quat::IDENTITY,
                Vec3::new(0.0, height / 2.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
        ),
        (
            // left
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::new(thicknes, height, 1.0),
                Quat::IDENTITY,
                Vec3::new(-width / 2.0, 0.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
        ),
        (
            // right
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::new(thicknes, height, 1.0),
                Quat::IDENTITY,
                Vec3::new(width / 2.0, 0.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::ONE),
        ),
    ]);
}
