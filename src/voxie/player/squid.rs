use glam::{Mat4, Quat, Vec3};

use crate::{
    collision::ColliderBody,
    renderer::{
        MESH_PROJECTILE, RenderMeshHandle,
        ecs_renderer::{MESH_CUBE, MESH_SQUID, RenderColor},
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
        MousePanConfig {
            last_mouse_position: (0.0, 0.0),
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
        PlayerMovement {
            speed: 15.0,
            input_velocity: Vec3::ZERO,
        },
        Gun {
            cooldown: 0.0,
            fire_rate: 2.5,
            triggered: false,
        },
    ));

    // Mesh entity: child of root
    let transform = Mat4::from_scale(Vec3::splat(0.15));
    world.spawn((
        LocalTransform { local: transform },
        Transform(transform),
        RenderMeshHandle(MESH_SQUID),
        RenderColor(Vec3::splat(0.85)),
        Parent(root),
    ));

    spawn_squid_capsule_collider(world, root);

    root
}

fn spawn_squid_capsule_collider(world: &mut hecs::World, root: hecs::Entity) {
    // Capsule collider independent from mesh transform
    let collider_transform = Mat4::from_scale_rotation_translation(
        // Scale required to visualize capsule, dosent affect collision body
        Vec3::new(1.0, 5.0, 1.0),
        // Align capsule with forward dir
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        // Offset towards forward dir
        Vec3::new(0.0, 0.0, -2.0),
    );
    world.spawn((
        LocalTransform {
            local: collider_transform,
        },
        Transform(collider_transform),
        RenderMeshHandle(MESH_CUBE),
        RenderColor(Vec3::X),
        VoxelCollider,
        ColliderBody::CapsuleCollider {
            radius: 0.5,
            height: 5.0,
        },
        Parent(root),
    ));
}

fn spawn_squid_sphere_collider(world: &mut hecs::World, root: hecs::Entity) {
    // Sphere collider independent from mesh transform
    let sphere_transform = Mat4::from_scale_rotation_translation(
        Vec3::splat(3.0),
        Quat::IDENTITY,
        // Offset towards forward dir
        Vec3::new(0.0, 0.0, -2.0),
    );
    world.spawn((
        LocalTransform {
            local: sphere_transform,
        },
        Transform(sphere_transform),
        RenderMeshHandle(MESH_PROJECTILE),
        RenderColor(Vec3::X),
        VoxelCollider,
        ColliderBody::SphereCollider { radius: 1.5 },
        Parent(root),
    ));
}
