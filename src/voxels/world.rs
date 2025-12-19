use log::{debug, error, info, trace};
use rayon::prelude::*;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::Instant,
};

use glam::{IVec3, Vec3};

use crate::{
    collision::{CollisionInfo, sphere_cast},
    octree::{AABB, IAabb, Octree, OctreeNodeIterator},
    voxels::{
        CHUNK_SIZE, Voxel, VoxelChunk,
        generators::{ChunkGenerator, cubic::CubicGenerator},
    },
};

use super::{VoxelKind, voxel::VoxelChunkIterator};

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

struct ChunkGenerationResult {
    position_octree_space: IVec3,
    chunk: VoxelChunk,
}

pub struct VoxelWorld {
    tree: Octree<Arc<VoxelChunk>>,
    generator: Arc<dyn ChunkGenerator>,

    // Channel for async chunk generation
    generated_chunk_receiver: Option<Receiver<Vec<ChunkGenerationResult>>>,
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
            tree,
            generated_chunk_receiver: None,
        }
    }

    pub fn get_size(&self) -> usize {
        self.tree.get_size()
    }

    /// Removes all voxels in a radius around the center.
    pub fn clear_sphere(&mut self, center: &Vec3, radius: f32) {
        // Query list of colliding voxels + their parent chunk
        let collider = IAabb::new(
            &IVec3::new(
                (center.x - radius / 2.0).round() as i32,
                (center.y - radius / 2.0).round() as i32,
                (center.z - radius / 2.0).round() as i32,
            ),
            radius.next_up() as usize,
        );
        let iter = self
            .iter_region_voxels_with_chunk(collider)
            // Solid
            .filter(|(voxel, _)| !matches!(voxel.kind, VoxelKind::Air))
            // Within radius
            .filter(|(voxel, _)| voxel.position.distance_squared(*center) < radius * radius);

        // Iterate and set voxel kind to Air to remove
        let mut voxels_removed = 0;
        for (voxel, chunk) in iter {
            let mut new_voxel = voxel;
            new_voxel.kind = VoxelKind::Air;
            chunk.insert(
                &IVec3::new(
                    voxel.position.x as i32,
                    voxel.position.y as i32,
                    voxel.position.z as i32,
                ),
                new_voxel,
            );
            voxels_removed += 1;
        }
        if voxels_removed > 0 {
            debug!("Removed {voxels_removed} colliding voxels ");
        }
    }

    #[cfg(test)]
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

    pub fn world_space_pos_to_chunk_space_pos(&self, world_space_pos: &Vec3) -> IVec3 {
        IVec3::new(
            (world_space_pos.x / CHUNK_SIZE as f32) as i32,
            (world_space_pos.y / CHUNK_SIZE as f32) as i32,
            (world_space_pos.z / CHUNK_SIZE as f32) as i32,
        )
    }

    /// Needs to be called every tick to insert generated chunks once generation is done
    pub fn receive_chunks(&mut self) {
        if self.generated_chunk_receiver.is_none() {
            // No thread running, nothing to do
            return;
        }
        let batch_channel = &self.generated_chunk_receiver.as_ref().unwrap();
        match batch_channel.try_recv() {
            Ok(chunks) => {
                debug!("Received {} chunks", chunks.len());
                for result in chunks {
                    self.tree
                        .insert(result.position_octree_space, Arc::new(result.chunk));
                }
                self.generated_chunk_receiver = None;
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => {
                // println!("Task still running...");
            }
            Err(err) => {
                error!("Task sender was dropped unexpectedly: {err}");
            }
        }
    }

    /// Checks world for uninitialized chunks within region. Should be called in regular intervals
    /// but not necessarily every tick
    fn spawn_chunk_generation(&mut self, region_world_space: IAabb, center: &Vec3) {
        const MAX_CHUNKS: usize = 200;
        if self.generated_chunk_receiver.is_some() {
            // Already running. Wait for finish first
            return;
        }
        let ivec_center = self.world_space_pos_to_chunk_space_pos(center);
        let mut all_empty_chunk_positions: Vec<IVec3> = self
            .iter_empty_chunk_positions(region_world_space)
            .collect::<Vec<IVec3>>();
        let size = all_empty_chunk_positions.len();
        if size == 0 {
            // Nothing to do
            return;
        } else {
            debug!("Found {size} uninitialized chunks ",);
        }
        let (tx, rx) = mpsc::channel();
        self.generated_chunk_receiver = Some(rx);
        let generator = Arc::clone(&self.generator);
        thread::spawn(move || {
            let mut generated_chunks: Vec<ChunkGenerationResult> = Vec::new();
            if size > MAX_CHUNKS {
                debug!("Max size exceeded. Sorting first...",);
                // If max size exceeded, we sort by distance to center point and only generate the first X
                all_empty_chunk_positions.sort_unstable_by(|a, b| {
                    a.distance_squared(ivec_center)
                        .partial_cmp(&b.distance_squared(ivec_center))
                        .unwrap()
                });
            }
            for chunk_origin in all_empty_chunk_positions.iter().take(MAX_CHUNKS) {
                let chunk_origin_world_space = chunk_origin * CHUNK_SIZE as i32;
                let chunk = generator.generate_chunk(chunk_origin_world_space);
                generated_chunks.push(ChunkGenerationResult {
                    position_octree_space: *chunk_origin,
                    chunk,
                });
            }
            debug!("Sending {size} chunks",);
            tx.send(generated_chunks).unwrap();
        });
    }

    pub fn expand_to_fit_region(&mut self, bounded_region: IAabb, center: &Vec3) {
        debug_assert!(bounded_region.min.x >= 0);
        debug_assert!(bounded_region.min.y >= 0);
        debug_assert!(bounded_region.min.z >= 0);
        let should_grow = !self
            .tree
            .get_total_region_world_space(CHUNK_SIZE)
            .contains(&bounded_region);
        // Grow tree if required
        if should_grow {
            info!("Growing world tree");
            self.tree.grow(CHUNK_SIZE);
        }
        self.spawn_chunk_generation(bounded_region, center);
    }

    pub fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("World")
            .size([300.0, 100.0], imgui::Condition::FirstUseEver)
            .position([900.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                let region = self.tree.get_total_region_world_space(CHUNK_SIZE);
                ui.text(format!("Total chunks: {}", self.get_size().pow(3)));
                ui.text(format!(
                    "Region covered; [{}] - [{}]",
                    region.min, region.max
                ));
                ui.text(format!(
                    "Generating: {}",
                    self.generated_chunk_receiver.is_some()
                ));
            });
    }

    pub fn iter_region_voxels_with_chunk(
        &self,
        region_world_space: IAabb,
    ) -> impl Iterator<Item = (Voxel, &Arc<VoxelChunk>)> {
        let bb_chunk_space = self.world_space_bb_to_chunk_space_bb(&region_world_space);
        let chunk_iterator = self.tree.iter_region(bb_chunk_space);
        VoxelWorldIterator {
            chunk_iterator,
            current_chunk: None,
            voxel_iterator: None,
            region: region_world_space,
        }
    }

    pub fn iter_region_voxels(&self, region_world_space: IAabb) -> impl Iterator<Item = Voxel> {
        self.iter_region_voxels_with_chunk(region_world_space)
            .map(|tuple| tuple.0)
    }

    pub fn iter_region_chunks(
        &self,
        region_world_space: &IAabb,
    ) -> OctreeNodeIterator<Arc<VoxelChunk>> {
        let bb_chunk_space = self.world_space_bb_to_chunk_space_bb(region_world_space);
        self.tree.iter_region(bb_chunk_space)
    }

    fn iter_empty_chunk_positions(&self, region_world_space: IAabb) -> impl Iterator<Item = IVec3> {
        let bb_chunk_space = self.world_space_bb_to_chunk_space_bb(&region_world_space);
        self.tree.iter_empty_within_region(bb_chunk_space)
    }

    pub fn query_sphere_cast(
        &self,
        origin: Vec3,
        radius: f32,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<CollisionInfo> {
        let start = Instant::now();
        // BB test
        let sphere_box_region_f = AABB::new(
            origin - radius * Vec3::ONE,
            origin + (radius + max_distance) * Vec3::ONE,
        );
        let sphere_box_region_i = IAabb::from(&sphere_box_region_f);
        let bbs = self
            .iter_region_voxels(sphere_box_region_i)
            .filter_map(|voxel| voxel.get_collider());
        let res = sphere_cast(origin, radius, direction, max_distance, bbs);
        trace!("Sphere cast took {}ms", start.elapsed().as_secs_f64() * 1e3);
        res
    }
}

pub struct VoxelWorldIterator<'a> {
    chunk_iterator: OctreeNodeIterator<'a, Arc<VoxelChunk>>,
    current_chunk: Option<&'a Arc<VoxelChunk>>,
    voxel_iterator: Option<VoxelChunkIterator<'a>>,
    /// Region in **world space**
    region: IAabb,
}

impl<'a> Iterator for VoxelWorldIterator<'a> {
    type Item = (Voxel, &'a Arc<VoxelChunk>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // If we have a current voxel iterator, try to yield from it
            if let Some(vox_iter) = self.voxel_iterator.as_mut() {
                if let Some(v) = vox_iter.next() {
                    return Some((v, self.current_chunk.unwrap()));
                }
            }

            // Current voxel iterator is exhausted; move to next chunk
            let next_chunk = self.chunk_iterator.next()?;
            self.current_chunk = Some(next_chunk);
            self.voxel_iterator = Some(next_chunk.iter_region(&self.region));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use glam::IVec3;

    use crate::{
        octree::IAabb,
        voxels::{CHUNK_SIZE, Voxel, VoxelWorld, generators::cubic::CubicGenerator},
    };

    use super::generate_chunk_world;

    #[test]
    fn test_chunk_generation() {
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
    fn test_chunk_region_size_2() {
        // 2x2x2 chunks
        let world = VoxelWorld::new_cubic(2);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(17, 1, 1));
        let chunks_in_octree: Vec<IVec3> = world
            .iter_region_chunks(&camera_bb_world_space)
            .map(|c| c.position)
            .collect();
        // camera bb overlaps with 2 chunks
        // 0,0,0 - 16,16,16
        // 17,0,0 - 32,15,15
        assert_eq!(chunks_in_octree.len(), 2);
    }

    #[test]
    fn test_chunk_region_size_4() {
        let world = VoxelWorld::new_cubic(4);
        let camera_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(16, 1, 1));
        let chunks_in_octree: Vec<IVec3> = world
            .iter_region_chunks(&camera_bb_world_space)
            .map(|c| c.position)
            .collect();
        // even though we now have 4x4x4 chunks, only 1 should overlap
        assert_eq!(chunks_in_octree.len(), 1);
    }

    #[test]
    fn test_voxel_region_query() {
        let world = VoxelWorld::new_cubic(1);
        let test_bb_world_space = IAabb::new_rect(IVec3::new(0, 0, 0), IVec3::new(2, 1, 1));
        let voxels: Vec<Voxel> = world.iter_region_voxels(test_bb_world_space).collect();
        // Cubes are centered around 0,0,0 , 0,0,1, etc...
        // So a BB from 0,0,0 to 2,1,1 will hit 3 voxels in x direction, 2 in y and 2 in z
        // -> 3*2*2 = 12
        assert_eq!(voxels.len(), 12);
    }
}
