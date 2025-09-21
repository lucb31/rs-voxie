use glam::IVec3;

#[derive(Clone)]
pub struct Voxel {
    // Transform
    pub position: IVec3,
}

impl Voxel {
    pub fn new() -> Voxel {
        let position = IVec3::ZERO;
        Self { position }
    }
}
