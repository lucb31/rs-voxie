use glam::{Mat4, Quat, Vec3};
use log::error;

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

struct SquidPivot {
    smoothened_tilt: f32,
}

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
            acceleration: 5.0,
            input_velocity: Vec3::ZERO,
        },
        Gun {
            cooldown: 0.0,
            fire_rate: 2.5,
            triggered: false,
        },
    ));

    let pivot = world.spawn((
        LocalTransform {
            local: Mat4::IDENTITY,
        },
        Transform(Mat4::IDENTITY),
        SquidPivot {
            smoothened_tilt: 0.0,
        },
        Parent(root),
    ));
    spawn_squid_mesh(world, pivot);
    spawn_squid_capsule_collider(world, pivot);

    root
}

/// Tilt player model according to movement
pub fn system_squid_velocity_tilt(world: &mut hecs::World, dt: f32) {
    let mut tilt = 0.0;
    let mut player_entity: Option<hecs::Entity> = None;
    // Input velocity based tilt. Not using actual velocity, because that is impacted by collision
    // events etc
    for (entity, movement) in world.query_mut::<&PlayerMovement>() {
        tilt = (movement.input_velocity).length_squared();
        player_entity = Some(entity);
    }
    if player_entity.is_none() {
        error!("Unable to apply tilt: No player entity found");
        return;
    }

    // Tilt pivot
    let tilt_animationspeed = 1.0;
    for (_, (pivot, local_transform)) in world
        .query::<(&mut SquidPivot, &mut LocalTransform)>()
        .iter()
    {
        // Move tilt linearly towards tilt from input
        pivot.smoothened_tilt = (pivot.smoothened_tilt
            + (tilt - 0.5).signum() * dt * tilt_animationspeed)
            .clamp(0.0, 1.0);

        // Interpolate rotation towards full / no tilt position
        let eased_rotation = -std::f32::consts::FRAC_PI_2 * ease_in_out_quad(pivot.smoothened_tilt);
        let (scale, _rot, local_translation) =
            local_transform.local.to_scale_rotation_translation();
        local_transform.local = Mat4::from_scale_rotation_translation(
            scale,
            Quat::from_rotation_x(eased_rotation),
            local_translation,
        )
    }
}

// Ease-in-out quadratic - smooth but quicker than cubic
fn ease_in_out_quad(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}

fn spawn_squid_mesh(world: &mut hecs::World, root: hecs::Entity) {
    let transform = Mat4::from_scale_rotation_translation(
        Vec3::splat(0.15),
        Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
        Vec3::ZERO,
    );
    world.spawn((
        LocalTransform { local: transform },
        Transform(transform),
        RenderMeshHandle(MESH_SQUID),
        RenderColor(Vec3::splat(0.85)),
        Parent(root),
    ));
}

fn spawn_squid_capsule_collider(world: &mut hecs::World, root: hecs::Entity) {
    // Capsule collider independent from mesh transform
    let collider_transform = Mat4::from_scale_rotation_translation(
        // Scale required to visualize capsule, dosent affect collision body
        Vec3::new(1.0, 5.0, 1.0),
        Quat::IDENTITY,
        // Offset towards forward dir
        Vec3::new(0.0, 2.0, 0.0),
    );
    world.spawn((
        LocalTransform {
            local: collider_transform,
        },
        Transform(collider_transform),
        //RenderMeshHandle(MESH_CUBE),
        RenderColor(Vec3::X),
        VoxelCollider,
        ColliderBody::CapsuleCollider {
            radius: 0.5,
            height: 5.0,
        },
        Parent(root),
    ));
}

fn _spawn_squid_sphere_collider(world: &mut hecs::World, root: hecs::Entity) {
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
