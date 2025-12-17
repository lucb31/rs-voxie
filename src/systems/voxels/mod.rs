use glam::{IVec3, Vec3};

use crate::{
    octree::IAabb,
    voxels::{CHUNK_SIZE, VoxelWorld},
};

/// Interims solution until voxel world is fully ecs based
/// Will query a region around the current player position to flag
/// world growth in that region
pub fn system_voxel_world_growth(voxel_world: &mut VoxelWorld, player_position: &Vec3) {
    let chunk_radius = 10;
    // Chunk-grid snapped camera pos
    let render_bb_min = IVec3::new(
        ((player_position.x / CHUNK_SIZE as f32) as i32 - chunk_radius) * CHUNK_SIZE as i32,
        ((player_position.y / CHUNK_SIZE as f32) as i32 - chunk_radius) * CHUNK_SIZE as i32,
        ((player_position.z / CHUNK_SIZE as f32) as i32 - chunk_radius) * CHUNK_SIZE as i32,
    );
    let render_bb_max = IVec3::new(
        ((player_position.x / CHUNK_SIZE as f32) as i32 + chunk_radius) * CHUNK_SIZE as i32,
        ((player_position.y / CHUNK_SIZE as f32) as i32 + chunk_radius) * CHUNK_SIZE as i32,
        ((player_position.z / CHUNK_SIZE as f32) as i32 + chunk_radius) * CHUNK_SIZE as i32,
    );
    let bb = IAabb::new_rect(render_bb_min, render_bb_max);
    // Query region to trigger 'should grow' flag
    voxel_world.query_region_chunks_with_init(&bb);
}
