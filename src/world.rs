use log::{debug, info, trace};
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
    voxel::{CHUNK_SIZE, Voxel, VoxelChunk},
    voxels::generators::{ChunkGenerator, cubic::CubicGenerator},
};

fn generate_chunk_world(
    tree_size: usize,
    generator: Arc<dyn ChunkGenerator>,
) -> Octree<Arc<VoxelChunk>> {
    info!("Generating world size {tree_size}");
    let start_world_generation = Instant::now();
    // Precalculate positions to be able to distribute them amongst worker threads
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
            let chunk = generator.generate_chunk(chunk_origin_world_space);

            // Update progress
            let prev = counter.fetch_add(1, Ordering::Relaxed);
            if prev % 1_000 == 0 || prev == total - 1 {
                let percent = (prev + 1) as f32 / total as f32 * 100.0;
                info!("{percent:.2}% done");
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
    info!(
        "World generation: Generated {} chunks in {}ms",
        world.get_all_depth_first().len(),
        start_world_generation.elapsed().as_secs_f32() * 1000.0,
    );
    world
}

pub struct VoxelWorld {
    tree: Octree<Arc<VoxelChunk>>,
    generator: Arc<dyn ChunkGenerator>,

    max_explored: Vec3,
}

impl VoxelWorld {
    /// Used mainly for testing purposes. Fills the entire world with the same voxel.
    #[allow(dead_code)]
    pub fn new_cubic(initial_size: usize) -> VoxelWorld {
        let generator: Arc<dyn ChunkGenerator> = Arc::new(CubicGenerator::new(CHUNK_SIZE));
        VoxelWorld::new(initial_size, generator)
    }

    pub fn new(initial_size: usize, generator: Arc<dyn ChunkGenerator>) -> VoxelWorld {
        let tree = generate_chunk_world(initial_size, generator.clone());
        Self {
            generator,
            max_explored: Vec3::ZERO,
            tree,
        }
    }

    pub fn get_size(&self) -> usize {
        self.tree.get_size()
    }

    /// Only use for small regions. Not very performant for bigger regions. Try using
    /// query_region_chunks instead
    pub fn query_region_voxels(&self, region_world_space: &IAabb) -> Vec<Voxel> {
        let start_query = Instant::now();
        let chunks_in_bb = self.query_region_chunks(region_world_space);
        let mut voxels_in_bb =
            Vec::with_capacity(chunks_in_bb.len() * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for chunk in &chunks_in_bb {
            chunk.query_region(region_world_space, &mut voxels_in_bb);
        }
        trace!(
            "Region query for region {:?} hit {} voxels in {} chunks. Took {}ms",
            region_world_space,
            voxels_in_bb.len(),
            chunks_in_bb.len(),
            start_query.elapsed().as_secs_f32() * 1000.0
        );
        voxels_in_bb
    }

    pub fn query_region_chunks(&self, region_world_space: &IAabb) -> Vec<Arc<VoxelChunk>> {
        let start_query = Instant::now();
        // Coarse-grained collision check using rounded IAabbs in chunk space
        let bb_chunk_space = self.world_space_bb_to_chunk_space_bb(region_world_space);
        let chunks = self.tree.query_region(&bb_chunk_space);
        // Fine-grained collision check using IAabbs in **world** space
        // Q: Why the additional BB check?
        // A: We have a pretty big rounding error since we need to round up to the next octree
        // coord when transforming the region in world space into octree space.
        // To get rid of all of the extra chunks, we filter again for intersection in WORLD space
        let intersecting_chunks: Vec<Arc<VoxelChunk>> = chunks
            .iter()
            .filter(|chunk| chunk.get_bb_i().intersects(region_world_space))
            .cloned()
            .collect();
        trace!(
            "Query returned {} chunks. Out of these {} are intersecting. Took {}ms",
            chunks.len(),
            intersecting_chunks.len(),
            start_query.elapsed().as_secs_f32() * 1000.0
        );
        intersecting_chunks
    }

    /// WARN: Should only be used for debugging.
    pub fn get_all_voxels(&self) -> Vec<Voxel> {
        let chunks = self.tree.get_all_depth_first();
        let mut voxels = Vec::with_capacity(chunks.len() * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for chunk in &chunks {
            voxels.extend_from_slice(chunk.voxel_slice());
        }
        voxels
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
    use std::sync::Arc;

    use glam::IVec3;

    use crate::{
        octree::IAabb, voxel::CHUNK_SIZE, voxels::generators::cubic::CubicGenerator,
        world::VoxelWorld,
    };

    use super::generate_chunk_world;

    #[test]
    fn test_chunk_world_size_2() {
        let generator = Arc::new(CubicGenerator::new(CHUNK_SIZE));
        let world = generate_chunk_world(2, generator);
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

    #[test]
    fn test_small_region_query() {
        let world = VoxelWorld::new_cubic(1);
        let test_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(2, 1, 1));
        let voxels = world.query_region_voxels(&test_bb_world_space);
        println!("{voxels:?}");
        // Cubes are centered around 0,0,0 , 0,0,1, etc...
        // So a BB from 0,0,0 to 2,1,1 will hit 3 voxels in x direction, 2 in y and 2 in z
        // -> 3*2*2 = 12
        assert_eq!(voxels.len(), 12);
    }
}
