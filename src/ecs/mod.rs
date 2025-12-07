use glam::{Mat4, Vec3};
use hecs::World;
use log::debug;

pub struct Transform(pub Mat4);
pub struct Velocity(pub Vec3);
pub struct Projectile;
pub struct Lifetime(pub f32);

pub fn system_movement(world: &mut World, dt: f32) {
    for (_entity, (transform, velocity)) in world.query_mut::<(&mut Transform, &Velocity)>() {
        transform.0.w_axis.x += velocity.0.x * dt;
        transform.0.w_axis.y += velocity.0.y * dt;
        transform.0.w_axis.z += velocity.0.z * dt;
    }
}

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
