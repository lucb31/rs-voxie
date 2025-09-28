use glam::Vec3;

use crate::octree::AABB;

#[derive(Clone, Debug)]
pub struct Voxel {
    pub position: Vec3,
    pub visible: bool,
}

impl Voxel {
    pub fn new() -> Voxel {
        let position = Vec3::ZERO;
        Self {
            position,
            visible: true,
        }
    }

    pub fn get_bb(&self) -> AABB {
        AABB::new(&self.position, 2.0)
    }
}
