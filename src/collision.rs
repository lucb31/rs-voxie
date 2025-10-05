use glam::Vec3;

use crate::octree::AABB;

#[derive(Debug)]
pub struct CollisionInfo {
    pub normal: Vec3,
    pub penetration_depth: f32,
}

pub fn get_sphere_aabb_collision_info(
    center: &Vec3,
    radius: f32,
    b: &AABB,
) -> Option<CollisionInfo> {
    let closest_point = center.clamp(b.min, b.max);
    let normal = center - closest_point;
    let length_sq = normal.length_squared();
    if length_sq > radius * radius {
        return None;
    }
    let penetration_depth = length_sq.sqrt();
    Some(CollisionInfo {
        normal,
        penetration_depth,
    })
}

