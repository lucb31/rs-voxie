use glam::{IVec3, Vec3};
use noise::{NoiseFn, Perlin};

use crate::voxel::{Voxel, VoxelChunk, VoxelKind};

use super::ChunkGenerator;

pub struct HeightmapGenerator {
    chunk_size: usize,
    height_limit: i32,
    perlin: Perlin,
}
impl HeightmapGenerator {
    pub fn new(chunk_size: usize) -> HeightmapGenerator {
        let seed: u32 = 99;
        Self {
            chunk_size,
            height_limit: 32,
            perlin: Perlin::new(seed),
        }
    }
}
impl ChunkGenerator for HeightmapGenerator {
    fn generate_chunk(&self, chunk_origin: IVec3) -> VoxelChunk {
        let mut chunk = VoxelChunk::new(chunk_origin);
        // TUNING
        let scale = 0.03;

        let mut nodes = 0;
        let lower_bound = chunk_origin;
        let upper_bound = chunk_origin + self.chunk_size as i32 * IVec3::ONE;
        let half = self.chunk_size as i32 / 2;
        let max_height = self.height_limit.min(half - 1) as f64;
        for x in lower_bound.x..upper_bound.x {
            let fx = x as f64 * scale;
            for z in lower_bound.z..upper_bound.z {
                let fz = z as f64 * scale;
                let noise_val = self.perlin.get([fx, fz]);
                let max_y = ((noise_val + 1.0) * (max_height / 2.0)).floor() as i32;
                // NOTE: As long as there is no way to 'dig down' into the world,
                // there is no point filling up the world below the surface voxels.
                // Once that is added we need to sample all 3d points or generate on the fly
                // -3 is to add SOME depth, otherwise there will be gaps in 'staircase' shapes
                for y in max_y - 3..max_y {
                    // This loop is not the most efficient, but prob does not matter here
                    if y < lower_bound.y {
                        continue;
                    } else if y > upper_bound.y - 1 {
                        continue;
                    }
                    let mut voxel = Voxel::new();
                    voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                    voxel.kind = VoxelKind::Dirt;
                    chunk.insert(&IVec3::new(x, y, z), voxel);
                    nodes += 1;
                }
            }
        }
        // println!("Chunk at {:?} produced {nodes} nodes", &chunk_origin);
        chunk
    }
}
