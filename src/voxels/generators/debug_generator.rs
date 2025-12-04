use glam::{IVec3, Vec3};

use crate::voxels::{Voxel, VoxelChunk, VoxelKind};

use super::ChunkGenerator;

pub struct DebugGenerator {
    chunk_size: usize,
}
impl DebugGenerator {
    pub fn new(chunk_size: usize) -> DebugGenerator {
        Self { chunk_size }
    }
}
impl ChunkGenerator for DebugGenerator {
    fn generate_chunk(&self, chunk_origin: IVec3) -> VoxelChunk {
        let chunk = VoxelChunk::new(chunk_origin);
        let size = self.chunk_size as i32 - 1;
        let positions = vec![
            chunk_origin,
            chunk_origin + IVec3::new(0, 0, size),
            chunk_origin + IVec3::new(0, size, 0),
            chunk_origin + IVec3::new(size, 0, 0),
            chunk_origin + IVec3::new(0, size, size),
            chunk_origin + IVec3::new(size, 0, size),
            chunk_origin + IVec3::new(size, size, 0),
            chunk_origin + IVec3::new(size, size, size),
        ];
        for pos in positions {
            let mut voxel = Voxel::new();
            voxel.position = Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32);
            voxel.kind = VoxelKind::Dirt;
            chunk.insert(&pos, voxel);
        }
        chunk
    }
}
