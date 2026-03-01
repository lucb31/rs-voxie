use glam::Vec3;
use hecs::Entity;

#[derive(Copy, Clone, Debug)]
pub struct CollisionInfo {
    pub normal: Vec3,
    pub contact_point: Vec3,
    pub penetration_depth: f32,
}

#[derive(Debug)]
pub struct CollisionEvent {
    pub info: CollisionInfo,
    pub a: Entity,
    /// If none, collided with voxel world
    pub b: Option<Entity>,
}

#[derive(Clone)]
pub enum ColliderBody {
    // assumes rect center equal to transform. Does not support offset
    AabbCollider { scale: Vec3 },
    // Assumes sphere center equal to transform
    SphereCollider { radius: f32 },
    // Assumes capsule center equal to transform. Height is cylinder portion only (excluding caps)
    CapsuleCollider { radius: f32, height: f32 },
}
