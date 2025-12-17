use std::time::Instant;

use glam::{Vec3, Vec4Swizzles};
use hecs::{Entity, World};
use log::trace;

use crate::{
    octree::{AABB, IAabb},
    systems::physics::Transform,
    voxels::VoxelWorld,
};

#[derive(Copy, Clone, Debug)]
pub struct CollisionInfo {
    pub normal: Vec3,
    pub contact_point: Vec3,
    pub distance: f32,
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Self { origin, direction }
    }

    /// Returns touple of (t_min, normal)
    pub fn intersect_aabb(&self, aabb: &AABB) -> Option<(f32, Vec3)> {
        // Helper closure to compute slab intersections safely
        fn slab(min: f32, max: f32, origin: f32, direction: f32) -> (f32, f32) {
            if direction != 0.0 {
                let inv_d = 1.0 / direction;
                let mut t0 = (min - origin) * inv_d;
                let mut t1 = (max - origin) * inv_d;
                if t0 > t1 {
                    std::mem::swap(&mut t0, &mut t1);
                }
                (t0, t1)
            } else {
                // Ray is parallel to this axis; check if origin is within slab
                if origin < min || origin > max {
                    (f32::INFINITY, -f32::INFINITY) // no intersection
                } else {
                    (-f32::INFINITY, f32::INFINITY) // always intersecting this slab
                }
            }
        }

        let (tx_min, tx_max) = slab(aabb.min.x, aabb.max.x, self.origin.x, self.direction.x);
        let (ty_min, ty_max) = slab(aabb.min.y, aabb.max.y, self.origin.y, self.direction.y);
        let (tz_min, tz_max) = slab(aabb.min.z, aabb.max.z, self.origin.z, self.direction.z);

        let t_min = tx_min.max(ty_min).max(tz_min);
        let t_max = tx_max.min(ty_max).min(tz_max);

        if t_max < t_min.max(0.0) {
            return None;
        }

        // Determine which axis contributed to t_min
        let normal = if t_min == tx_min {
            if self.direction.x < 0.0 {
                Vec3::new(1.0, 0.0, 0.0)
            } else {
                Vec3::new(-1.0, 0.0, 0.0)
            }
        } else if t_min == ty_min {
            if self.direction.y < 0.0 {
                Vec3::new(0.0, 1.0, 0.0)
            } else {
                Vec3::new(0.0, -1.0, 0.0)
            }
        } else {
            if self.direction.z < 0.0 {
                Vec3::new(0.0, 0.0, 1.0)
            } else {
                Vec3::new(0.0, 0.0, -1.0)
            }
        };
        Some((t_min, normal))
    }

    pub fn intersects_aabb_within_t(&self, other: &AABB, t_max: f32) -> Option<CollisionInfo> {
        let (t, normal) = self.intersect_aabb(other)?;
        if t <= t_max {
            let hit_point = self.origin + self.direction * t;
            return Some(CollisionInfo {
                normal,
                contact_point: hit_point,
                distance: t,
            });
        }
        None
    }
}

fn get_sphere_aabb_collision_info(center: &Vec3, radius: f32, b: &AABB) -> Option<CollisionInfo> {
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

pub enum VoxelCollider {
    SphereCollider { radius: f32 },
}

pub struct CollisionEvent {
    pub info: CollisionInfo,
    pub a: Entity,
    /// If none, collided with voxel world
    pub b: Option<Entity>,
}

pub fn system_voxel_world_collisions(
    world: &mut World,
    voxel_world: &VoxelWorld,
) -> Vec<CollisionEvent> {
    let mut all_collisions: Vec<CollisionEvent> = Vec::new();
    for (_entity, (transform, collider)) in world.query::<(&Transform, &VoxelCollider)>().iter() {
        match collider {
            VoxelCollider::SphereCollider { radius } => {
                let center = transform.0.w_axis.xyz();
                all_collisions.extend(iter_sphere_collision(voxel_world, center, *radius).map(
                    |info| CollisionEvent {
                        info,
                        a: _entity,
                        b: None,
                    },
                ));
            }
        };
    }
    all_collisions
}

pub fn iter_sphere_collision(
    world: &VoxelWorld,
    center: Vec3,
    radius: f32,
) -> impl Iterator<Item = CollisionInfo> {
    debug_assert!(center.is_finite());
    debug_assert!(radius > 0.00001);
    // BB test
    let sphere_box_region_f = AABB::new_center(&center, radius * 2.0);
    let sphere_box_region_i = IAabb::from(&sphere_box_region_f);
    // WARN: Known issue: When an oject is coming from **negative** x,y,z values
    // we will not return correct voxels in the region check. More specifically this
    // should only happen at the 'edge' of the world.
    // Accepted risk
    let iter = world.iter_region_voxels(&sphere_box_region_i);
    iter.filter_map(move |voxel| {
        let vox_collider = voxel.get_collider()?;
        get_sphere_aabb_collision_info(&center, radius, &vox_collider)
    })
}

fn sphere_cast(
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

pub fn query_sphere_cast(
    world: &VoxelWorld,
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
    let bbs = world
        .iter_region_voxels(&sphere_box_region_i)
        .filter_map(|voxel| voxel.get_collider());
    let res = sphere_cast(origin, radius, direction, max_distance, bbs);
    trace!("Sphere cast took {}ms", start.elapsed().as_secs_f64() * 1e3);
    res
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{
        collision::{CollisionInfo, iter_sphere_collision, sphere_cast},
        octree::AABB,
        voxels::VoxelWorld,
    };

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
    fn test_sphere_collision_origin() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::ZERO;
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions: Vec<CollisionInfo> =
            iter_sphere_collision(&world, sphere_position, sphere_radius).collect();
        assert_eq!(collisions.len(), 1);
    }
    #[test]
    fn test_sphere_collision_offset_minimal_x() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::new(-0.45, 0.0, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions: Vec<CollisionInfo> =
            iter_sphere_collision(&world, sphere_position, sphere_radius).collect();
        assert_eq!(collisions.len(), 1);
    }
    #[test]
    fn test_sphere_collision_offset_y() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::new(0.0, 0.1, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions: Vec<CollisionInfo> =
            iter_sphere_collision(&world, sphere_position, sphere_radius).collect();
        assert_eq!(collisions.len(), 2);
    }
    #[test]
    fn test_sphere_collision_offset_x() {
        let world = VoxelWorld::new_cubic(1);
        // Offset in y direction, should collide with 2 voxel
        let sphere_position = Vec3::new(0.5, 0.0, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions: Vec<CollisionInfo> =
            iter_sphere_collision(&world, sphere_position, sphere_radius).collect();
        assert_eq!(collisions.len(), 2);
    }
    #[test]
    fn test_sphere_collision_offset_yx() {
        let world = VoxelWorld::new_cubic(1);
        let sphere_position = Vec3::new(0.5, 0.5, 0.0);
        // Avoid rounding errors
        let sphere_radius = 0.49;
        let collisions: Vec<CollisionInfo> =
            iter_sphere_collision(&world, sphere_position, sphere_radius).collect();
        assert_eq!(collisions.len(), 4);
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
