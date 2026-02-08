use glam::{Mat4, Quat, Vec3};

use crate::{
    collision::ColliderBody,
    renderer::{
        MESH_PROJECTILE, RenderMeshHandle,
        ecs_renderer::{MESH_SQUID, RenderColor},
    },
    systems::{
        gun::Gun,
        physics::{LocalTransform, Parent, Transform, Velocity},
    },
    voxels::VoxelCollider,
};

use super::{MousePanConfig, Player, PlayerMovement};

pub fn spawn_squid(world: &mut hecs::World, position: Vec3) -> hecs::Entity {
    // Root entity: controls movement, mouse rotation
    let root = world.spawn((
        Player,
        LocalTransform {
            local: Mat4::from_translation(position),
        },
        Transform(Mat4::from_translation(position)),
        Velocity(Vec3::ZERO),
        VoxelCollider,
        //ColliderBody::SphereCollider { radius: 0.5 },
        ColliderBody::CapsuleCollider {
            radius: 0.5,
            height: 9.0,
        },
        MousePanConfig {
            last_mouse_position: (0.0, 0.0),
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
        PlayerMovement { speed: 15.0 },
        Gun {
            cooldown: 0.0,
            fire_rate: 2.5,
            triggered: false,
        },
    ));

    // Mesh entity: child of root, static 180° Y rotation
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::splat(0.15),
        Quat::from_rotation_x(std::f32::consts::PI / 2.0),
        Vec3::ZERO,
    );
    world.spawn((
        LocalTransform { local: transform },
        Transform(transform),
        RenderMeshHandle(MESH_SQUID),
        RenderColor(Vec3::splat(0.85)),
        Parent(root),
    ));

    // Collider entity: for reference
    let collider_transform = Mat4::from_scale(Vec3::splat(1.0));
    world.spawn((
        LocalTransform {
            local: collider_transform,
        },
        Transform(collider_transform),
        RenderMeshHandle(MESH_PROJECTILE),
        RenderColor(Vec3::X),
        Parent(root),
    ));

    root
}
