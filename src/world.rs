use noise::{NoiseFn, Perlin};
use rayon::prelude::*;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

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
    let positions: Vec<(usize, usize, usize)> = (0..tree_size)
        .flat_map(|x| (0..tree_size).flat_map(move |y| (0..tree_size).map(move |z| (x, y, z))))
        .collect();

    let counter = Arc::new(AtomicUsize::new(0));
    let total = tree_size * tree_size * tree_size;
    let chunks: Vec<(IVec3, Arc<VoxelChunk>)> = positions
        .into_par_iter()
        .map(|(x, y, z)| {
            let chunk_origin_world_space = IVec3::new(
                (x * CHUNK_SIZE) as i32,
                (y * CHUNK_SIZE) as i32,
                (z * CHUNK_SIZE) as i32,
            );

            let chunk = match mode {
                WorldGenerationMode::Cubic => generate_chunk_cubic(chunk_origin_world_space),
                WorldGenerationMode::Perlin3D => generate_chunk_3d_noise(chunk_origin_world_space),
                WorldGenerationMode::PerlinHeightmap => {
                    generate_chunk_heightmap(chunk_origin_world_space)
                }
            };

            // Update progress
            let prev = counter.fetch_add(1, Ordering::Relaxed);
            if prev % 1_000 == 0 || prev == total - 1 {
                let percent = (prev + 1) as f32 / total as f32 * 100.0;
                println!("{:.2}% done", percent);
            }

            let pos = IVec3::new(x as i32, y as i32, z as i32);
            (pos, Arc::new(chunk))
        })
        .collect();

    // Insert all chunks into the world
    let mut world: Octree<Arc<VoxelChunk>> = Octree::new(tree_size);
    for (pos, chunk) in chunks {
        world.insert(pos, chunk);
    }
    println!(
        "took {}ms for {} chunks",
        start_world_generation.elapsed().as_secs_f32() * 1000.0,
        world.get_all_depth_first().len()
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
                // [-1; 1]
                let noise_val = perlin.get([fx, fy, fz]);
                // Noise band -> Hollow caves
                if noise_val > 0.1 && noise_val < 0.25 {
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

fn flatten_chunks(chunks: &[Arc<VoxelChunk>]) -> Vec<Voxel> {
    let start_flattening_chunks = Instant::now();
    let voxels_per_chunk = CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE;
    let total_voxels = voxels_per_chunk * chunks.len();

    let mut result = Vec::with_capacity(total_voxels);

    for chunk in chunks {
        let slice = chunk.voxel_slice();
        result.extend_from_slice(slice);
    }

    println!(
        "Flattening chunks took {}ms",
        start_flattening_chunks.elapsed().as_secs_f32() * 1000.0
    );
    result
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

    // @deprecated: Use the query_region_chunks method instead and filter / interate
    // over voxels as late as possible
    pub fn query_region_voxels(&self, region_world_space: &IAabb) -> Vec<Voxel> {
        let start_query = Instant::now();
        let chunks = self.query_region_chunks(region_world_space);
        println!(
            "RegionQuery: chunks took {}ms",
            start_query.elapsed().as_secs_f32() * 1000.0
        );
        let voxels_in_bb_region = flatten_chunks(&chunks);
        println!(
            "Total region query for region {:?} hit {} voxels. Took {}ms",
            region_world_space,
            voxels_in_bb_region.len(),
            start_query.elapsed().as_secs_f32() * 1000.0
        );
        voxels_in_bb_region
    }

    pub fn query_region_chunks(&self, region_world_space: &IAabb) -> Vec<Arc<VoxelChunk>> {
        let bb_chunk_space = self.world_space_bb_to_chunk_space_bb(region_world_space);
        let chunks = self.tree.query_region(&bb_chunk_space);
        // NOTE: Q: Why the additional BB check?
        // A: We have a pretty big rounding error since we need to round up to the next octree
        // coord when transforming the region in world space into octree space.
        // To get rid of all of the extra chunks, we filter again for intersection in WORLD space
        let intersecting_chunks: Vec<Arc<VoxelChunk>> = chunks
            .iter()
            .filter(|chunk| chunk.get_bb().intersects(region_world_space))
            .cloned()
            .collect();
        println!(
            "Query returned {} chunks. Out of these {} are intersecting",
            chunks.len(),
            intersecting_chunks.len()
        );
        intersecting_chunks
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

    use crate::{octree::IAabb, voxel::CHUNK_SIZE, world::VoxelWorld};

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
        // camera bb overlaps with 2 chunks
        // 0,0,0 - 15,15,15
        // 16,0,0 - 31,15,15
        assert_eq!(chunks_in_octree.len(), 2);
    }

    #[test]
    fn test_chunk_world_size_4_region_query() {
        let world = VoxelWorld::new_cubic(4);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(16, 1, 1));
        let chunks_in_octree = world.query_region_chunks(&camera_bb_world_space);
        // even though we now have 4x4x4 chunks, only 2 should overlap
        assert_eq!(chunks_in_octree.len(), 2);
    }

    #[test]
    fn test_chunk_world_size_4_region_query_bb_variation() {
        let world = VoxelWorld::new_cubic(4);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(17, 1, 1));
        let chunks_in_octree = world.query_region_chunks(&camera_bb_world_space);
        // even though we now have 4x4x4 chunks, only 2 should overlap
        assert_eq!(chunks_in_octree.len(), 2);
    }
}
