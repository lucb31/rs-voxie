use glam::{Mat4, Quat, Vec3};
use hecs::{Entity, World};

use crate::systems::physics::Transform;

use crate::collision::ColliderBody;

pub(super) struct PongBallTrigger {
    pub(super) player_slot: usize,
}
struct PongBoundary;

/// Not networked at all. Completely static scene asset
pub(super) fn spawn_boundaries(world: &mut World, width: f32, height: f32) {
    let thicknes = 0.25;
    let horizontal_scale = Vec3::new(width - thicknes * 1.01, thicknes, 1.0);
    let vertical_scale = Vec3::new(thicknes, height, 1.0);
    let mut entities: Vec<Entity> = world
        .spawn_batch([
            (
                // top
                Transform(Mat4::from_scale_rotation_translation(
                    horizontal_scale,
                    Quat::IDENTITY,
                    Vec3::new(0.0, -height / 2.0, 0.0),
                )),
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
                ColliderBody::AabbCollider {
                    scale: horizontal_scale,
                },
                PongBoundary,
            ),
        ])
        .map(|b| b)
        .collect();
    world
        .spawn_batch([
            (
                // left
                Transform(Mat4::from_scale_rotation_translation(
                    vertical_scale,
                    Quat::IDENTITY,
                    Vec3::new(-width / 2.0, 0.0, 0.0),
                )),
                ColliderBody::AabbCollider {
                    scale: vertical_scale,
                },
                PongBallTrigger { player_slot: 0 },
                PongBoundary,
            ),
            (
                // right
                Transform(Mat4::from_scale_rotation_translation(
                    vertical_scale,
                    Quat::IDENTITY,
                    Vec3::new(width / 2.0, 0.0, 0.0),
                )),
                ColliderBody::AabbCollider {
                    scale: vertical_scale,
                },
                PongBallTrigger { player_slot: 1 },
                PongBoundary,
            ),
        ])
        .map(|b| b)
        .for_each(|e| entities.push(e));
    #[cfg(feature = "gui")]
    {
        // Add rendering components
        let mut commands = hecs::CommandBuffer::new();
        let render_mesh_handle =
            crate::renderer::RenderMeshHandle(crate::renderer::ecs_renderer::MESH_CUBE);
        let render_color = crate::renderer::ecs_renderer::RenderColor(Vec3::ONE);
        for entity in entities {
            commands.insert(entity, (render_mesh_handle.clone(), render_color.clone()));
        }
        commands.run_on(world);
    }
}
