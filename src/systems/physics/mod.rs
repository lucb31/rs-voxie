use glam::{Mat4, Vec3};
use hecs::World;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transform(pub Mat4);
pub struct Velocity(pub Vec3);

pub fn system_movement(world: &mut World, dt: f32) {
    for (_entity, (transform, velocity)) in world.query_mut::<(&mut Transform, &Velocity)>() {
        transform.0.w_axis.x += velocity.0.x * dt;
        transform.0.w_axis.y += velocity.0.y * dt;
        transform.0.w_axis.z += velocity.0.z * dt;
    }
}
