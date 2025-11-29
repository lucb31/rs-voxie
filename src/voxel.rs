use glam::{IVec3, Vec3};

use crate::octree::{AABB, IAabb};

#[derive(Copy, Clone, Debug)]
pub enum VoxelKind {
    Dirt,
    Air,
}

#[derive(Copy, Clone, Debug)]
pub struct Voxel {
    pub position: Vec3,
    pub kind: VoxelKind,
}

impl Voxel {
    pub fn new() -> Voxel {
        let position = Vec3::ZERO;
        Self {
            position,
            kind: VoxelKind::Air,
        }
    }

    pub fn get_collider(&self) -> AABB {
        // Air voxels should already be filtered out at an earlier stage
        debug_assert!(!matches!(self.kind, VoxelKind::Air));
        AABB::new_center(&self.position, 1.0)
    }
}

#[derive(Debug)]
pub struct VoxelChunk {
    pub voxels: Box<[[[Voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>, // owned, contiguous memory
    // Minimum corner (world pos)
    pub position: IVec3,
}

pub const CHUNK_SIZE: usize = 16;
impl VoxelChunk {
    // New chunk at **world_pos**
    pub fn new(position: IVec3) -> VoxelChunk {
        let voxels = Box::new(
            [(); CHUNK_SIZE]
                .map(|_| [(); CHUNK_SIZE].map(|_| [(); CHUNK_SIZE].map(|_| Voxel::new()))),
        );
        Self { position, voxels }
    }

    pub fn insert(&mut self, world_pos: &IVec3, voxel: Voxel) {
        let relative_pos = world_pos - self.position;
        debug_assert!(
            relative_pos.x >= 0,
            "relative_pos out of bounds {relative_pos}"
        );
        debug_assert!(relative_pos.y >= 0);
        debug_assert!(relative_pos.z >= 0);
        let x = relative_pos.x as usize;
        let y = relative_pos.y as usize;
        let z = relative_pos.z as usize;
        debug_assert!(x < CHUNK_SIZE);
        debug_assert!(y < CHUNK_SIZE);
        debug_assert!(z < CHUNK_SIZE);
        self.voxels[x][y][z] = voxel;
    }

    /// Returns flattened list of voxels
    pub fn voxel_slice(&self) -> &[Voxel] {
        let ptr = self.voxels.as_ptr() as *const Voxel;
        let len = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        // SAFETY: We know voxels are stored contiguously in Box
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    pub fn get_bb_i(&self) -> IAabb {
        IAabb::new(&self.position, CHUNK_SIZE)
    }

    // Will only check indices within overlap
    pub fn query_region(&self, bbi_world_space: &IAabb, res: &mut Vec<Voxel>) {
        let chunk_bb = self.get_bb_i();
        let optional_overlap = chunk_bb.intersection(bbi_world_space);
        if let Some(overlap) = optional_overlap {
            let min_x = (overlap.min.x - 1 - self.position.x).max(0) as usize;
            let max_x = (overlap.max.x + 1 - self.position.x).min(CHUNK_SIZE as i32 - 1) as usize;
            let min_y = (overlap.min.y - 1 - self.position.y).max(0) as usize;
            let max_y = (overlap.max.y + 1 - self.position.y).min(CHUNK_SIZE as i32 - 1) as usize;
            let min_z = (overlap.min.z - 1 - self.position.z).max(0) as usize;
            let max_z = (overlap.max.z + 1 - self.position.z).min(CHUNK_SIZE as i32 - 1) as usize;
            for x in min_x..max_x {
                for y in min_y..max_y {
                    for z in min_z..max_z {
                        let voxel = self.voxels[x][y][z];
                        // Ignore air voxels
                        if matches!(voxel.kind, VoxelKind::Air) {
                            continue;
                        }
                        // No need to do another BB test. Already tested by chunk overlap
                        res.push(voxel);
                    }
                }
            }
        }
    }

    pub fn iter_voxels(&self) -> impl Iterator<Item = (IVec3, Voxel)> + '_ {
        (0..CHUNK_SIZE).flat_map(move |z| {
            (0..CHUNK_SIZE).flat_map(move |y| {
                (0..CHUNK_SIZE).filter_map(move |x| {
                    let voxel = self.voxels[x][y][z];
                    match voxel.kind {
                        VoxelKind::Air => None,
                        VoxelKind::Dirt => {
                            let pos = self.position + IVec3::new(x as i32, y as i32, z as i32);
                            Some((pos, voxel))
                        }
                    }
                })
            })
        })
    }
}
