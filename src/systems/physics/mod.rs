use glam::{Mat4, Vec3};
use hecs::World;
use hierarchy_cache::HierarchyCache;
use log::error;
use serde::{Deserialize, Serialize};

pub mod hierarchy_cache;

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Transform in **world** coordinates
pub struct Transform(pub Mat4);
pub struct Velocity(pub Vec3);

/// Transform in **local** coordinates (relative to parent node)
pub struct LocalTransform {
    pub local: Mat4,
}

pub struct Parent(pub hecs::Entity);

// Update hierarchical transforms
pub fn system_update_world_transforms(world: &mut hecs::World, cache: &mut HierarchyCache) {
    // Update cache
    if cache.is_dirty {
        cache.rebuild(world);
    }

    // Iterate sorted entity hierarchy
    for (_depth, entity) in &cache.entities_by_depth {
        // Check if entity has a parent
        match (
            world.get::<&Parent>(*entity),
            world.get::<&LocalTransform>(*entity),
        ) {
            (Ok(parent), Ok(local_transform)) => {
                // Child node: compute world transform from parent
                if let Ok(parent_transform) = world.get::<&Transform>(parent.0) {
                    if let Ok(mut world_transform) = world.get::<&mut Transform>(*entity) {
                        world_transform.0 = parent_transform.0 * local_transform.local;
                    }
                } else {
                    error!(
                        "Transform hierarchy defined, but could not find parent entity {:?}",
                        parent.0
                    );
                }
            }
            _ => {
                // Root node or entity without both Parent and LocalTransform
                // World transform remains unchanged (or could be computed from LocalTransform)
            }
        }
    }
}

// Variant of movement system that supports nested entity hierarchies
pub fn system_movement_with_hierarchy_nodes(
    world: &mut World,
    dt: f32,
    hierarchy_cache: &mut HierarchyCache,
) {
    system_update_world_transforms(world, hierarchy_cache);
    system_movement(world, dt);
}

pub fn system_movement(world: &mut World, dt: f32) {
    for (_entity, (transform, velocity)) in world.query_mut::<(&mut Transform, &Velocity)>() {
        transform.0.w_axis.x += velocity.0.x * dt;
        transform.0.w_axis.y += velocity.0.y * dt;
        transform.0.w_axis.z += velocity.0.z * dt;
    }
}
