use glam::Vec3;

use crate::octree::AABB;

use super::CollisionInfo;

pub(super) struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub(super) fn new(origin: Vec3, direction: Vec3) -> Ray {
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
