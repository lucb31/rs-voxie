use glam::{IVec3, Vec3};

use crate::octree::AABB;

#[derive(Clone, Debug)]
pub struct Voxel {
    pub position: Vec3,
}

impl Voxel {
    pub fn new() -> Voxel {
        let position = Vec3::ZERO;
        Self { position }
    }
}
