use glam::IVec3;

use crate::voxel::VoxelChunk;

pub mod cubic;
pub mod heightmap;
pub mod noise3d;

pub trait ChunkGenerator: Sync + Send {
    /// Generates voxel chunk for given origin position in **world** space
    fn generate_chunk(&self, chunk_origin: IVec3) -> VoxelChunk;
}
