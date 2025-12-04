use glam::{IVec3, Vec3};
use log::trace;
use noise::{NoiseFn, Perlin};

use crate::voxels::{Voxel, VoxelChunk, VoxelKind};

use super::ChunkGenerator;

pub struct Noise3DGenerator {
    chunk_size: usize,
    perlin: Perlin,
    scale: f64,
}
impl Noise3DGenerator {
    pub fn new(chunk_size: usize) -> Noise3DGenerator {
        let seed: u32 = 99;
        Self {
            chunk_size,
            perlin: Perlin::new(seed),
            scale: 0.03,
        }
    }
}
impl ChunkGenerator for Noise3DGenerator {
    fn generate_chunk(&self, chunk_origin: IVec3) -> VoxelChunk {
        let mut chunk = VoxelChunk::new(chunk_origin);
        let lower_bound = chunk_origin;
        let upper_bound = chunk_origin + self.chunk_size as i32 * IVec3::ONE;
        let mut nodes = 0;

        // TUNING
        for x in lower_bound.x..upper_bound.x {
            let fx = x as f64 * self.scale;
            for z in lower_bound.z..upper_bound.z {
                let fz = z as f64 * self.scale;
                for y in lower_bound.y..upper_bound.y {
                    let fy = y as f64 * self.scale;
                    // [-1; 1]
                    let noise_val = self.perlin.get([fx, fy, fz]);
                    // Noise band -> Hollow caves
                    if noise_val > 0.1 && noise_val < 0.25 {
                        let mut voxel = Voxel::new();
                        voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                        if noise_val < 0.15 {
                            voxel.kind = VoxelKind::Granite;
                        } else if noise_val < 0.2 {
                            voxel.kind = VoxelKind::Coal;
                        } else {
                            voxel.kind = VoxelKind::Sand;
                        }
                        chunk.insert(&IVec3::new(x, y, z), voxel);
                        nodes += 1;
                    }
                }
            }
        }
        trace!(
            "Produces noise 3d chunk at {:?} with {nodes} nodes",
            &chunk_origin
        );
        chunk
    }
}
