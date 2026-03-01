use glam::{Mat4, Vec3};
use hecs::World;
use log::error;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Transform in **world** coordinates
pub struct Transform(pub Mat4);
pub struct Velocity(pub Vec3);

/// Transform in **local** coordinates (relative to parent node)
pub struct LocalTransform {
    pub local: Mat4,
}

pub struct Parent(pub hecs::Entity);

// WARN: This is a simple system, that is not guaranteed to work for deep hierarchy structures
fn system_update_world_transforms(world: &mut hecs::World) {
    for (_entity, (parent, local_transform, world_transform)) in world
        .query::<(&Parent, &LocalTransform, &mut Transform)>()
        .iter()
    {
        if let Ok(parent_transform) = world.get::<&Transform>(parent.0) {
            world_transform.0 = parent_transform.0 * local_transform.local;
        } else {
            error!(
                "Transform hierarchy defined, but could not find parent entity {:?}",
                parent.0
            )
        }
    }
}

pub fn system_movement(world: &mut World, dt: f32) {
    // Ensure hierarchical transforms are updated first
    system_update_world_transforms(world);
    for (_entity, (transform, velocity)) in world.query_mut::<(&mut Transform, &Velocity)>() {
        transform.0.w_axis.x += velocity.0.x * dt;
        transform.0.w_axis.y += velocity.0.y * dt;
        transform.0.w_axis.z += velocity.0.z * dt;
    }
}
