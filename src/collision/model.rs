use glam::Vec3;
use hecs::Entity;

#[derive(Copy, Clone, Debug)]
pub struct CollisionInfo {
    pub normal: Vec3,
    pub contact_point: Vec3,
    pub distance: f32,
}

pub struct CollisionEvent {
    pub info: CollisionInfo,
    pub a: Entity,
    /// If none, collided with voxel world
    pub b: Option<Entity>,
}
