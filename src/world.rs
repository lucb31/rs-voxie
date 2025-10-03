use noise::{NoiseFn, Perlin};
use std::{sync::Arc, time::Instant};

use glam::{IVec3, Vec3};

use crate::{
    octree::{IAabb, Octree},
    voxel::{CHUNK_SIZE, Voxel, VoxelChunk, VoxelKind},
};

enum WorldGenerationMode {
    Cubic,
    PerlinHeightmap,
    Perlin3D,
}

fn generate_chunk_world(tree_size: usize, mode: WorldGenerationMode) -> Octree<Arc<VoxelChunk>> {
    println!("Generating world size {tree_size}");
    let start_world_generation = Instant::now();
    // 2x2x2 world tree that houses 8 chunks with a dimension of 16 x 16 x 16
    let mut world: Octree<Arc<VoxelChunk>> = Octree::new(tree_size);
    for x in 0..tree_size {
        for y in 0..tree_size {
            for z in 0..tree_size {
                let chunk_world_origin = IVec3::new(
                    (x * CHUNK_SIZE) as i32,
                    (y * CHUNK_SIZE) as i32,
                    (z * CHUNK_SIZE) as i32,
                );
                let mut chunk_opt: Option<VoxelChunk> = None;
                match mode {
                    WorldGenerationMode::Cubic => {
                        chunk_opt = Some(generate_chunk_cubic(chunk_world_origin));
                    }
                    WorldGenerationMode::Perlin3D => {
                        chunk_opt = Some(generate_chunk_3d_noise(chunk_world_origin));
                    }
                    WorldGenerationMode::PerlinHeightmap => {
                        chunk_opt = Some(generate_chunk_heightmap(chunk_world_origin));
                    }
                }
                if let Some(chunk) = chunk_opt {
                    world.insert(IVec3::new(x as i32, y as i32, z as i32), Arc::new(chunk));
                }
            }
        }
        println!("... {}% done", x as f32 / tree_size as f32 * 100.0);
    }
    println!(
        "took {}ms",
        start_world_generation.elapsed().as_secs_f32() * 1000.0
    );
    world
}

fn generate_chunk_cubic(chunk_origin: IVec3) -> VoxelChunk {
    let mut chunk = VoxelChunk::new(chunk_origin);
    let lower_bound = chunk_origin;
    let upper_bound = chunk_origin + CHUNK_SIZE as i32 * IVec3::ONE;
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

fn generate_chunk_heightmap(chunk_origin: IVec3) -> VoxelChunk {
    const HEIGHT_LIMIT: i32 = 32;

    let mut chunk = VoxelChunk::new(chunk_origin);
    // TUNING
    const SEED: u32 = 99;
    let scale = 0.03;
    let perlin = Perlin::new(SEED);

    let mut nodes = 0;
    let lower_bound = chunk_origin;
    let upper_bound = chunk_origin + CHUNK_SIZE as i32 * IVec3::ONE;
    let half = CHUNK_SIZE as i32 / 2;
    let max_height = HEIGHT_LIMIT.min(half - 1) as f64;
    for x in lower_bound.x..upper_bound.x {
        let fx = x as f64 * scale;
        for z in lower_bound.z..upper_bound.z {
            let fz = z as f64 * scale;
            let noise_val = perlin.get([fx, fz]);
            let max_y = ((noise_val + 1.0) * (max_height / 2.0)).floor() as i32;
            // NOTE: As long as there is no way to 'dig down' into the world,
            // there is no point filling up the world below the surface voxels.
            // Once that is added we need to sample all 3d points or generate on the fly
            // -3 is to add SOME depth, otherwise there will be gaps in 'staircase' shapes
            for y in max_y - 3..max_y {
                // This loop is not the most efficient, but prob does not matter here
                if y < lower_bound.y {
                    //println!("This is not the right chunk for us. y too low");
                    continue;
                } else if y > upper_bound.y - 1 {
                    //println!("This is not the right chunk for us. y too high");
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

// Generate chunk with origin in world coordinates
fn generate_chunk_3d_noise(chunk_origin: IVec3) -> VoxelChunk {
    let mut chunk = VoxelChunk::new(chunk_origin);
    let lower_bound = chunk_origin;
    let upper_bound = chunk_origin + CHUNK_SIZE as i32 * IVec3::ONE;
    let mut nodes = 0;

    // TUNING
    const SEED: u32 = 99;
    let scale = 0.03;
    let perlin = Perlin::new(SEED);
    for x in lower_bound.x..upper_bound.x {
        let fx = x as f64 * scale;
        for z in lower_bound.z..upper_bound.z {
            let fz = z as f64 * scale;
            for y in lower_bound.y..upper_bound.y {
                let fy = y as f64 * scale;
                let noise_val = perlin.get([fx, fy, fz]);
                if noise_val > 0.0 {
                    let mut voxel = Voxel::new();
                    voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                    voxel.kind = VoxelKind::Dirt;
                    chunk.insert(&IVec3::new(x, y, z), voxel);
                    nodes += 1;
                }
            }
        }
    }
    // println!("Chunk at {:?} produced {nodes} nodes", &chunk_origin);
    chunk
}

pub struct VoxelWorld {
    tree: Octree<Arc<VoxelChunk>>,
}

impl VoxelWorld {
    /// Used mainly for testing purposes. Fills the entire world with the same voxel.
    #[allow(dead_code)]
    pub fn new_cubic(initial_size: usize) -> VoxelWorld {
        let tree = generate_chunk_world(initial_size, WorldGenerationMode::Cubic);
        Self { tree }
    }

    pub fn new(initial_size: usize) -> VoxelWorld {
        let tree = generate_chunk_world(initial_size, WorldGenerationMode::Perlin3D);
        Self { tree }
    }

    pub fn get_size(&self) -> usize {
        self.tree.get_size()
    }

    pub fn query_region_voxels(&self, region_world_space: &IAabb) -> Vec<Voxel> {
        let start_query = Instant::now();
        let chunks = self.query_region_chunks(region_world_space);
        let mut voxels_in_bb_region: Vec<Voxel> = vec![];
        for chunk in &chunks {
            voxels_in_bb_region.extend(chunk.query_region(region_world_space));
        }
        println!(
            "Region query for region {:?} hit {} voxels. Took {}ms",
            region_world_space,
            voxels_in_bb_region.len(),
            start_query.elapsed().as_secs_f32() * 1000.0
        );
        voxels_in_bb_region
    }

    fn query_region_chunks(&self, region_world_space: &IAabb) -> Vec<Arc<VoxelChunk>> {
        let bb_chunk_space = self.world_space_bb_to_chunk_space_bb(region_world_space);
        self.tree.query_region(&bb_chunk_space)
    }

    fn world_space_bb_to_chunk_space_bb(&self, world_space_bb: &IAabb) -> IAabb {
        IAabb::new_rect(
            IVec3::new(
                (world_space_bb.min.x as f32 / CHUNK_SIZE as f32).floor() as i32,
                (world_space_bb.min.y as f32 / CHUNK_SIZE as f32).floor() as i32,
                (world_space_bb.min.z as f32 / CHUNK_SIZE as f32).floor() as i32,
            ),
            IVec3::new(
                (world_space_bb.max.x as f32 / CHUNK_SIZE as f32).ceil() as i32,
                (world_space_bb.max.y as f32 / CHUNK_SIZE as f32).ceil() as i32,
                (world_space_bb.max.z as f32 / CHUNK_SIZE as f32).ceil() as i32,
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use glam::IVec3;

    use crate::{
        octree::IAabb,
        voxel::{CHUNK_SIZE, Voxel},
        world::VoxelWorld,
    };

    use super::generate_chunk_world;

    #[test]
    fn test_chunk_world_size_2() {
        let world = generate_chunk_world(2, crate::world::WorldGenerationMode::Cubic);
        let chunks = world.get_all_depth_first();
        // Size 2 -> 8 chunks
        assert_eq!(chunks.len(), 8);

        // 8 chunks a 16x16x16 voxels
        let mut total_voxels = 0;
        for chunk in &chunks {
            for (_, _) in chunk.iter_voxels() {
                total_voxels += 1;
            }
        }
        assert_eq!(
            total_voxels,
            chunks.len() * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE
        );
    }

    #[test]
    fn test_chunk_world_size_2_region_query() {
        // 2x2x2 chunks
        let world = VoxelWorld::new_cubic(2);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(16, 1, 1));
        let chunks_in_octree = world.query_region_chunks(&camera_bb_world_space);
        // camera bb barely overlaps with all chunks (border overlap)
        assert_eq!(chunks_in_octree.len(), 8);

        let voxels_in_bb_region: Vec<Voxel> = world.query_region_voxels(&camera_bb_world_space);
        assert_eq!(voxels_in_bb_region.len(), 16);
    }

    #[test]
    fn test_chunk_world_size_4_region_query() {
        let world = VoxelWorld::new_cubic(4);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(16, 1, 1));
        let chunks_in_octree = world.query_region_chunks(&camera_bb_world_space);
        // even though we now have 4x4x4 chunks, only 8 should overlap
        assert_eq!(chunks_in_octree.len(), 8);

        let voxels_in_bb_region: Vec<Voxel> = world.query_region_voxels(&camera_bb_world_space);
        assert_eq!(voxels_in_bb_region.len(), 16);
    }

    #[test]
    fn test_chunk_world_size_4_region_query_bb_variation() {
        let world = VoxelWorld::new_cubic(4);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(17, 1, 1));
        let voxels_in_bb_region: Vec<Voxel> = world.query_region_voxels(&camera_bb_world_space);
        assert_eq!(voxels_in_bb_region.len(), 17);
    }
}
