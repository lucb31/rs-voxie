use hecs::World;
use log::debug;

use crate::{collision::CollisionEvent, voxels::VoxelWorld};

pub struct Projectile;
pub struct Lifetime(pub f32);

pub fn system_lifetime(world: &mut World, dt: f32) {
    let mut to_delete = Vec::new();
    for (entity, lifetime) in world.query_mut::<&mut Lifetime>() {
        lifetime.0 -= dt;
        if lifetime.0 <= 0.0 {
            debug!("Entity {entity:?} reached the end of its lifetime");
            to_delete.push(entity);
        }
    }
    for entity in to_delete {
        world
            .despawn(entity)
            .expect("Could not delete scheduled entity");
    }
}

pub fn system_projectile_collisions(
    world: &mut World,
    voxel_world: &mut VoxelWorld,
    collision_events: &[CollisionEvent],
) {
    for collision in collision_events {
        if world.get::<&Projectile>(collision.a).is_ok() {
            // Projectile involved
            debug!(
                "Projectile hit the world at {}. Removing",
                collision.info.contact_point
            );
            world
                .despawn(collision.a)
                .expect("Unable to remove projectile");
            // Explosion
            let explosion_radius = 3.0;
            voxel_world.clear_sphere(&collision.info.contact_point, explosion_radius);
        }
    }
}
