use std::time::Instant;

use glam::{IVec3, Vec3};

use crate::octree::IAabb;

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
}

#[derive(Debug)]
pub struct VoxelChunk {
    pub voxels: Box<[[[Voxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE]>, // owned, contiguous memory
    // Minimum corner (world pos)
    position: IVec3,
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

    pub fn voxel_slice(&self) -> &[Voxel] {
        let ptr = self.voxels.as_ptr() as *const Voxel;
        let len = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
        // SAFETY: We know voxels are stored contiguously in Box
        unsafe { std::slice::from_raw_parts(ptr, len) }
    }

    pub fn get_bb(&self) -> IAabb {
        IAabb::new(&self.position, CHUNK_SIZE)
    }

    pub fn query_region(&self, bb_world_space: &IAabb) -> Vec<Voxel> {
        let chunk_bb = self.get_bb();
        let optional_overlap = chunk_bb.intersection(bb_world_space);
        if let Some(overlap) = optional_overlap {
            let mut tested_voxels = 0;
            let mut res = Vec::with_capacity(overlap.area() as usize);
            for x in overlap.min.x..overlap.max.x {
                for y in overlap.min.y..overlap.max.y {
                    for z in overlap.min.z..overlap.max.z {
                        tested_voxels += 1;
                        let idx_x = (x - self.position.x) as usize;
                        let idx_y = (y - self.position.y) as usize;
                        let idx_z = (z - self.position.z) as usize;
                        let voxel = self.voxels[idx_x][idx_y][idx_z];
                        // Ignore air voxels
                        if matches!(voxel.kind, VoxelKind::Air) {
                            continue;
                        }
                        // No need to do another BB test. Already tested by chunk overlap
                        res.push(voxel);
                    }
                }
            }
            //            println!(
            //                "Chunk tested {} voxels and hit {}",
            //                tested_voxels,
            //                res.len()
            //            );
            res
        } else {
            // println!("No overlap. Can skip chunk");
            vec![]
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
