use glam::{IVec3, Vec3};

use crate::voxels::{Voxel, VoxelChunk, VoxelKind};

use super::ChunkGenerator;

pub struct CubicGenerator {
    chunk_size: usize,
}
impl CubicGenerator {
    pub fn new(chunk_size: usize) -> CubicGenerator {
        Self { chunk_size }
    }
}
impl ChunkGenerator for CubicGenerator {
    fn generate_chunk(&self, chunk_origin: IVec3) -> VoxelChunk {
        let mut chunk = VoxelChunk::new(chunk_origin);
        let lower_bound = chunk_origin;
        let upper_bound = chunk_origin + self.chunk_size as i32 * IVec3::ONE;
        for x in lower_bound.x..upper_bound.x {
            for y in lower_bound.y..upper_bound.y {
                for z in lower_bound.z..upper_bound.z {
                    let mut voxel = Voxel::new();
                    voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                    voxel.kind = VoxelKind::Dirt;
                    chunk.insert(&IVec3::new(x, y, z), voxel);
                }
            }
        }
        chunk
    }
}
