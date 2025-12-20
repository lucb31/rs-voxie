use glam::Vec3;

use crate::octree::AABB;

use super::CollisionInfo;

fn aabb_penetration(a: &AABB, b: &AABB) -> Vec3 {
    let px = (b.max.x - a.min.x).min(a.max.x - b.min.x);
    let py = (b.max.y - a.min.y).min(a.max.y - b.min.y);
    let pz = (b.max.z - a.min.z).min(a.max.z - b.min.z);
    Vec3::new(px, py, pz)
}

fn aabb_collision_normal(a: &AABB, b: &AABB) -> Vec3 {
    let penetration = aabb_penetration(a, b);

    if penetration.x < penetration.y && penetration.x < penetration.z {
        if a.max.x - b.min.x < b.max.x - a.min.x {
            Vec3::new(-1.0, 0.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        }
    } else if penetration.y < penetration.z {
        if a.max.y - b.min.y < b.max.y - a.min.y {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        }
    } else {
        if a.max.z - b.min.z < b.max.z - a.min.z {
            Vec3::new(0.0, 0.0, -1.0)
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }
}

fn aabb_contact_point(a: &AABB, b: &AABB) -> Vec3 {
    let overlap_min = a.min.max(b.min); // component-wise max
    let overlap_max = a.max.min(b.max); // component-wise min
    (overlap_min + overlap_max) * 0.5
}

pub fn get_aabb_aabb_collision_info(a: &AABB, b: &AABB) -> Option<CollisionInfo> {
    match a.intersects(b) {
        true => {
            let normal = aabb_collision_normal(a, b);
            let contact_point = aabb_contact_point(a, b);
            let penetration = aabb_penetration(a, b);

            // Optionally, penetration depth along normal:
            let penetration_depth = penetration.dot(normal.abs()); // scalar
            Some(CollisionInfo {
                normal,
                contact_point,
                penetration_depth,
            })
        }
        false => None,
    }
}
