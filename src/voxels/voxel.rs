use std::sync::{
    RwLock,
    atomic::{AtomicBool, Ordering},
};

use glam::{IVec3, Vec3};

use crate::octree::{AABB, IAabb};

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum VoxelKind {
    Coal = 0,
    Granite = 1,
    Dirt = 2,
    Sand = 3,
    Air = 99,
}

impl VoxelKind {
    pub fn material_index(self) -> u32 {
        self as u32
    }
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
    voxels: RwLock<Box<[[[Voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>>, // owned, contiguous memory
    /// Minimum corner (world pos)
    pub position: IVec3,
    is_dirty: AtomicBool,
}

// TODO: Would be cleaner to have this as a world parameter
pub const CHUNK_SIZE: usize = 16;

impl VoxelChunk {
    // New chunk at **world_pos**
    pub fn new(position: IVec3) -> VoxelChunk {
        let voxels = Box::new(
            [(); CHUNK_SIZE]
                .map(|_| [(); CHUNK_SIZE].map(|_| [(); CHUNK_SIZE].map(|_| Voxel::new()))),
        );
        Self {
            is_dirty: AtomicBool::new(true),
            position,
            voxels: RwLock::new(voxels),
        }
    }

    pub fn set_clean(&self) {
        self.is_dirty.store(false, Ordering::Relaxed);
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty.load(Ordering::Relaxed)
    }

    pub fn insert(&self, world_pos: &IVec3, voxel: Voxel) {
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
        self.voxels.write().unwrap()[x][y][z] = voxel;
        self.is_dirty.store(true, Ordering::Relaxed);
    }

    /// Returns flattened list of voxels
    pub fn voxel_slice(&self) -> &[Voxel] {
        let ptr = self.voxels.read().unwrap().as_ptr() as *const Voxel;
        let len = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        // SAFETY: We know voxels are stored contiguously in Box
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    pub fn get_bb_i(&self) -> IAabb {
        IAabb::new(&self.position, CHUNK_SIZE)
    }

    pub fn query_region(&self, bbi_world_space: &IAabb, res: &mut Vec<Voxel>) {
        res.extend(
            self.iter_region(bbi_world_space)
                // Backwards compatibility: Only visible chunks
                .filter(|v| !matches!(v.kind, VoxelKind::Air)),
        );
    }

    #[cfg(test)]
    pub fn iter_voxels(&self) -> impl Iterator<Item = (IVec3, Voxel)> + '_ {
        (0..CHUNK_SIZE).flat_map(move |z| {
            (0..CHUNK_SIZE).flat_map(move |y| {
                (0..CHUNK_SIZE).filter_map(move |x| {
                    let voxel = self.voxels.read().unwrap()[x][y][z];
                    match voxel.kind {
                        VoxelKind::Air => None,
                        _ => {
                            let pos = self.position + IVec3::new(x as i32, y as i32, z as i32);
                            Some((pos, voxel))
                        }
                    }
                })
            })
        })
    }

    pub fn iter_region(&self, region_world_space: &IAabb) -> VoxelChunkIterator {
        let chunk_bb = self.get_bb_i();
        let optional_overlap = chunk_bb.intersection(region_world_space);
        // Will only check indices within overlap
        if let Some(overlap) = optional_overlap {
            let min_x = (overlap.min.x - 1 - self.position.x).max(0) as usize;
            let max_x = (overlap.max.x + 1 - self.position.x).min(CHUNK_SIZE as i32 - 1) as usize;
            let min_y = (overlap.min.y - 1 - self.position.y).max(0) as usize;
            let max_y = (overlap.max.y + 1 - self.position.y).min(CHUNK_SIZE as i32 - 1) as usize;
            let min_z = (overlap.min.z - 1 - self.position.z).max(0) as usize;
            let max_z = (overlap.max.z + 1 - self.position.z).min(CHUNK_SIZE as i32 - 1) as usize;
            VoxelChunkIterator {
                x: min_x,
                y: min_y,
                z: min_z,
                min_y,
                min_z,
                max_x,
                max_y,
                max_z,
                chunk: self,
            }
        } else {
            VoxelChunkIterator {
                x: 0,
                y: 0,
                z: 0,
                min_y: 0,
                min_z: 0,
                max_x: 0,
                max_y: 0,
                max_z: 0,
                chunk: self,
            }
        }
    }
}

pub struct VoxelChunkIterator<'a> {
    x: usize,
    y: usize,
    z: usize,
    min_y: usize,
    min_z: usize,
    max_x: usize,
    max_y: usize,
    max_z: usize,
    chunk: &'a VoxelChunk,
}

impl<'a> Iterator for VoxelChunkIterator<'a> {
    type Item = Voxel;

    fn next(&mut self) -> Option<Self::Item> {
        while self.x < self.max_x {
            while self.y < self.max_y {
                while self.z < self.max_z {
                    let x = self.x;
                    let y = self.y;
                    let z = self.z;
                    self.z += 1;
                    let voxel = self.chunk.voxels.read().unwrap()[x][y][z];
                    return Some(voxel);
                }
                self.z = self.min_z;
                self.y += 1;
            }
            self.y = self.min_y;
            self.x += 1;
        }
        None
    }
}

#[cfg(test)]
mod test {
    use glam::IVec3;

    use crate::{
        octree::IAabb,
        voxels::{CHUNK_SIZE, VoxelChunk},
    };

    use super::{Voxel, VoxelKind};

    #[test]
    fn query_region_basic_overlap() {
        let chunk = VoxelChunk::new(IVec3::ZERO);

        // Place solid voxels
        chunk.voxels.write().unwrap()[1][1][1] = solid_voxel();
        chunk.voxels.write().unwrap()[2][2][2] = solid_voxel();

        // Query region fully inside chunk
        let region = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(3, 3, 3));

        let mut res = Vec::new();
        chunk.query_region(&region, &mut res);

        assert_eq!(res.len(), 2);
    }

    #[test]
    fn query_region_no_overlap() {
        let mut chunk = VoxelChunk::new(IVec3::ZERO);

        chunk.voxels.write().unwrap()[1][1][1] = solid_voxel();

        let region = IAabb::new_rect(IVec3::new(100, 100, 100), IVec3::new(110, 110, 110));

        let mut res = Vec::new();
        chunk.query_region(&region, &mut res);

        assert!(res.is_empty());
    }

    #[test]
    fn query_region_partial_overlap_at_edge() {
        let mut chunk = VoxelChunk::new(IVec3::ZERO);

        chunk.voxels.write().unwrap()[0][0][0] = solid_voxel();
        chunk.voxels.write().unwrap()[CHUNK_SIZE - 1][0][0] = solid_voxel();

        let region = IAabb::new_rect(IVec3::new(-2, -2, -2), IVec3::new(1, 1, 1));

        let mut res = Vec::new();
        chunk.query_region(&region, &mut res);

        assert_eq!(res.len(), 1);
    }

    #[test]
    fn query_region_touching_but_not_overlapping() {
        let mut chunk = VoxelChunk::new(IVec3::ZERO);

        chunk.voxels.write().unwrap()[0][0][0] = solid_voxel();

        let region = IAabb::new_rect(
            IVec3::new(CHUNK_SIZE as i32, 0, 0),
            IVec3::new(CHUNK_SIZE as i32 + 5, 5, 5),
        );

        let mut res = Vec::new();
        chunk.query_region(&region, &mut res);

        assert!(res.is_empty());
    }

    #[test]
    fn query_region_only_air_voxels() {
        let chunk = VoxelChunk::new(IVec3::ZERO);

        let region = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(4, 4, 4));

        let mut res = Vec::new();
        chunk.query_region(&region, &mut res);

        assert!(res.is_empty());
    }

    fn solid_voxel() -> Voxel {
        let mut voxel = Voxel::new();
        voxel.kind = VoxelKind::Dirt;
        voxel
    }
}
