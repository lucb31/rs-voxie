use glam::Vec3;

use crate::{collision::ray::Ray, octree::AABB};

use super::CollisionInfo;

pub fn get_sphere_aabb_collision_info(
    center: &Vec3,
    radius: f32,
    b: &AABB,
) -> Option<CollisionInfo> {
    let closest_point = center.clamp(b.min, b.max);
    let normal = center - closest_point;
    let length_sq = normal.length_squared();
    if length_sq > radius * radius {
        return None;
    }
    let mut distance = 0.0;
    // Avoid NaN issues with very small distances
    if length_sq >= 1e-5 {
        distance = length_sq.sqrt();
    }
    Some(CollisionInfo {
        contact_point: closest_point,
        normal: normal.normalize(),
        distance,
    })
}

pub fn sphere_cast(
    origin: Vec3,
    radius: f32,
    direction: Vec3,
    max_distance: f32,
    boxes: impl Iterator<Item = AABB>,
) -> Option<CollisionInfo> {
    let dir = direction.normalize();
    let mut closest_hit: Option<CollisionInfo> = None;

    let ray = Ray::new(origin, dir);
    for aabb in boxes {
        // Inflate AABB by sphere radius
        let inflated = AABB::new(aabb.min - Vec3::ONE * radius, aabb.max + Vec3::ONE * radius);
        if let Some(collision_info) = ray.intersects_aabb_within_t(&inflated, max_distance) {
            if closest_hit.is_none() || collision_info.distance < closest_hit.unwrap().distance {
                closest_hit = Some(collision_info);
            }
        }
    }
    debug_assert!(closest_hit?.distance <= max_distance);
    closest_hit
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{collision::sphere_cast, octree::AABB, voxels::VoxelWorld};

    use super::get_sphere_aabb_collision_info;

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
            let bb = voxel.get_collider().unwrap();
            let collision_test = get_sphere_aabb_collision_info(&center, radius, &bb);
            if collision_test.is_some() {
                colliders += 1;
            }
        }
        assert_eq!(colliders, 1);
    }

    #[test]
    fn test_sphere_cast_hits_plane() {
        let plane = AABB::new(
            Vec3::new(-100.0, -100.0, 5.0),
            Vec3::new(100.0, 100.0, 25.0),
        );
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let radius = 1.0;
        let max_distance = 10.0;

        let hit = sphere_cast(origin, radius, direction, max_distance, [plane].into_iter());

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert!(
            (hit.contact_point.z - 4.0).abs() < 1e-5,
            "Wrong contact point z {}",
            hit.contact_point.z
        );
        assert!((hit.distance - 4.0).abs() < 1e-5);
    }
    #[test]
    fn test_sphere_cast_respects_max_dist() {
        // Plane at z=12 (1 above max_dist + radius)
        let plane = AABB::new(
            Vec3::new(-100.0, -100.0, 12.0),
            Vec3::new(100.0, 100.0, 25.0),
        );
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(0.0, 0.0, 1.0);
        let radius = 1.0;
        let max_distance = 10.0;

        let hit = sphere_cast(origin, radius, direction, max_distance, [plane].into_iter());

        assert!(hit.is_none());
    }
    #[test]
    fn test_sphere_cast_grazing_hit() {
        let bb = AABB::new(Vec3::new(1.0, -1.0, 4.0), Vec3::new(3.0, 1.0, 6.0));
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 1.0).normalize();
        let radius = 1.0;
        let max_distance = 10.0;

        let hit = sphere_cast(origin, radius, direction, max_distance, [bb].into_iter());

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert!(
            (hit.contact_point.z - 3.0).abs() < 1e-5,
            "Wrong contact point z {}",
            hit.contact_point.z
        );
    }
    #[test]
    fn test_sphere_cast_center_missese_shell_hits() {
        let bb = AABB::new_center(&Vec3::new(2.0, 0.0, 5.0), 1.0);
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 1.0).normalize();
        let radius = 1.5;
        let max_distance = 10.0;

        let hit = sphere_cast(origin, radius, direction, max_distance, [bb].into_iter());

        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert!(
            (hit.contact_point.z - 3.0).abs() < 1e-5,
            "Wrong contact point z {}",
            hit.contact_point.z
        );
    }
}
