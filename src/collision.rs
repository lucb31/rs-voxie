use std::time::Instant;

use glam::Vec3;

use crate::{
    octree::{AABB, IAabb},
    world::VoxelWorld,
};

#[derive(Debug)]
pub struct CollisionInfo {
    pub normal: Vec3,
    pub penetration_depth: f32,
    pub contact_point: Vec3,
}

fn get_sphere_aabb_collision_info(center: &Vec3, radius: f32, b: &AABB) -> Option<CollisionInfo> {
    let closest_point = center.clamp(b.min, b.max);
    let normal = center - closest_point;
    let length_sq = normal.length_squared();
    if length_sq > radius * radius || length_sq <= 0.001 {
        // NOTE: Ignore length close to 0; will cause NaN errors otherwise.
        return None;
    }
    let penetration_depth = length_sq.sqrt();
    Some(CollisionInfo {
        contact_point: closest_point,
        normal,
        penetration_depth,
    })
}

pub fn query_sphere_collision(
    world: &VoxelWorld,
    center: &Vec3,
    radius: f32,
) -> Vec<CollisionInfo> {
    let start = Instant::now();
    // BB test
    let sphere_box_region_f = AABB::new_center(center, radius * 2.0);
    let sphere_box_region_i = IAabb::from(&sphere_box_region_f);
    // WARN: Known issue: When an oject is coming from **negative** x,y,z values
    // we will not return correct voxels in the region check. More specifically this
    // should only happen at the 'edge' of the world.
    // Accepted risk
    let voxels = world.query_region_voxels(&sphere_box_region_i);
    // Collision test
    let mut collisions = Vec::with_capacity(voxels.len());
    for voxel in &voxels {
        let vox_collider = voxel.get_collider();
        let collision_info = get_sphere_aabb_collision_info(center, radius, &vox_collider);
        if let Some(info) = collision_info {
            collisions.push(info);
        }
    }
    println!(
        "Collision test took {}ms, tested {} voxels",
        start.elapsed().as_secs_f32() * 1e3,
        voxels.len(),
    );
    collisions
    //    self.sma_collision_check_time
    //        .add(start.elapsed().as_secs_f32() * 1e6);
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{octree::AABB, world::VoxelWorld};

    use super::{get_sphere_aabb_collision_info, query_sphere_collision};

    #[test]
    fn test_simple_sphere_bb() {
        let center = Vec3::new(-1.0, 0.0, 0.0);
        let radius = 0.5;
        let bb = AABB::new_center(&Vec3::ZERO, 1.0);
        let collision_test = get_sphere_aabb_collision_info(&center, radius, &bb);
        assert!(collision_test.is_some());
    }

    #[test]
    fn test_simple_sphere_bb_miss() {
        let center = Vec3::new(-1.1, 0.0, 0.0);
        let radius = 0.5;
        let bb = AABB::new_center(&Vec3::ZERO, 1.0);
        let collision_test = get_sphere_aabb_collision_info(&center, radius, &bb);
        assert!(collision_test.is_none());
    }

    #[test]
    fn test_simple_sphere_bb_without_region_check() {
        // Run without region query to test isolated
        let world = VoxelWorld::new_cubic(1);
        let center = Vec3::new(-1.0, 0.0, 0.0);
        let radius = 0.5;
        let voxels = world.get_all_voxels();
        let mut colliders = 0;
        for voxel in &voxels {
            let bb = voxel.get_collider();
            let collision_test = get_sphere_aabb_collision_info(&center, radius, &bb);
            if collision_test.is_some() {
                colliders += 1;
            }
        }
        assert_eq!(colliders, 1);
    }

    #[test]
    fn test_sphere_collision_origin() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::ZERO;
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions = query_sphere_collision(&world, &sphere_position, sphere_radius);
        assert_eq!(collisions.len(), 1);
    }
    #[test]
    fn test_sphere_collision_offset_minimal_x() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::new(-0.45, 0.0, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions = query_sphere_collision(&world, &sphere_position, sphere_radius);
        assert_eq!(collisions.len(), 1);
    }
    #[test]
    fn test_sphere_collision_offset_y() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::new(0.0, 0.1, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions = query_sphere_collision(&world, &sphere_position, sphere_radius);
        assert_eq!(collisions.len(), 2);
    }
    #[test]
    fn test_sphere_collision_offset_x() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::new(0.5, 0.0, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions = query_sphere_collision(&world, &sphere_position, sphere_radius);
        assert_eq!(collisions.len(), 2);
    }
    #[test]
    fn test_sphere_collision_offset_yx() {
        let world = VoxelWorld::new_cubic(1);
        let sphere_position = Vec3::new(0.5, 0.5, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions = query_sphere_collision(&world, &sphere_position, sphere_radius);
        assert_eq!(collisions.len(), 4);
    }
}
