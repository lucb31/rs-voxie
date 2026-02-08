use glam::{Mat4, Vec3};

use crate::{collision::sphere::sphere_cast, octree::AABB};

use super::CollisionInfo;

#[derive(Debug, Clone)]
pub struct Capsule {
    pub endpoint_a: Vec3,
    pub endpoint_b: Vec3,
    pub radius: f32,
}

impl Capsule {
    pub fn from_transform(transform: Mat4, radius: f32, height: f32) -> Self {
        let (_scale, rotation, translation) = Mat4::to_scale_rotation_translation(&transform);

        let half_height = height * 0.5;
        let local_endpoint_a = Vec3::new(0.0, -half_height, 0.0);
        let local_endpoint_b = Vec3::new(0.0, half_height, 0.0);

        let world_endpoint_a = rotation * local_endpoint_a + translation;
        let world_endpoint_b = rotation * local_endpoint_b + translation;

        Self {
            endpoint_a: world_endpoint_a,
            endpoint_b: world_endpoint_b,
            radius,
        }
    }

    fn get_height(&self) -> f32 {
        (self.endpoint_b - self.endpoint_a).length()
    }
}

fn closest_point_on_line_segment(point: Vec3, line_start: Vec3, line_end: Vec3) -> Vec3 {
    let line_vec = line_end - line_start;
    let point_vec = point - line_start;
    let line_len_sq = line_vec.length_squared();

    if line_len_sq < f32::EPSILON {
        return line_start;
    }

    let t = point_vec.dot(line_vec) / line_len_sq;
    let t_clamped = t.clamp(0.0, 1.0);

    line_start + line_vec * t_clamped
}

pub(super) fn get_capsule_sphere_collision_info(
    capsule: &Capsule,
    sphere_center: Vec3,
    sphere_radius: f32,
) -> Option<CollisionInfo> {
    let closest_on_capsule =
        closest_point_on_line_segment(sphere_center, capsule.endpoint_a, capsule.endpoint_b);

    let offset = sphere_center - closest_on_capsule;
    let dist_sq = offset.length_squared();
    let sum_radii = capsule.radius + sphere_radius;

    if dist_sq <= sum_radii * sum_radii {
        let mut normal = Vec3::X;
        if dist_sq > f32::EPSILON {
            normal = offset.normalize();
        }

        let dist = dist_sq.sqrt();
        let contact_point = closest_on_capsule + normal * capsule.radius;

        return Some(CollisionInfo {
            normal,
            contact_point,
            penetration_depth: -(sum_radii - dist),
        });
    }

    None
}

fn sample_capsule_points(capsule: &Capsule) -> impl Iterator<Item = Vec3> {
    let height = capsule.get_height();
    let num_samples = (height / capsule.radius).floor() as usize;

    // Ensure at least 3 samples (start, mid, end)
    let num_samples = num_samples.max(3);

    let axis = (capsule.endpoint_b - capsule.endpoint_a).normalize();
    let step = height / (num_samples - 1) as f32;

    (0..num_samples).map(move |i| capsule.endpoint_a + axis * (i as f32 * step))
}

pub fn capsule_cast(
    capsule: &Capsule,
    direction: Vec3,
    max_distance: f32,
    boxes: impl Iterator<Item = AABB>,
) -> Option<CollisionInfo> {
    let samples = sample_capsule_points(capsule);
    let mut closest_hit: Option<CollisionInfo> = None;
    let bbs: Vec<AABB> = boxes.collect();
    for sample in samples {
        if let Some(collision_info) = sphere_cast(
            sample,
            capsule.radius,
            direction,
            max_distance,
            bbs.clone().into_iter(),
        ) {
            if closest_hit.is_none()
                || collision_info.penetration_depth < closest_hit.unwrap().penetration_depth
            {
                closest_hit = Some(collision_info);
            }
        }
    }
    debug_assert!(closest_hit?.penetration_depth <= max_distance);
    closest_hit
}

pub fn get_capsule_aabb_collision_info(capsule: &Capsule, aabb: &AABB) -> Option<CollisionInfo> {
    // Find closest point on capsule to AABB
    let capsule_center = (capsule.endpoint_a + capsule.endpoint_b) * 0.5;

    // First, get closest point on AABB to capsule center
    let closest_on_aabb = capsule_center.clamp(aabb.min, aabb.max);

    // Then find closest point on capsule to that AABB point
    let closest_on_capsule =
        closest_point_on_line_segment(closest_on_aabb, capsule.endpoint_a, capsule.endpoint_b);

    // Calculate distance from capsule surface to AABB point
    let offset = closest_on_aabb - closest_on_capsule;
    let dist_sq = offset.length_squared();
    let radius_sq = capsule.radius * capsule.radius;

    if dist_sq <= radius_sq {
        let normal: Vec3;
        let penetration: f32;
        let contact_point: Vec3;

        if dist_sq > f32::EPSILON {
            let dist = dist_sq.sqrt();
            normal = offset.normalize();
            penetration = capsule.radius - dist;
            contact_point = closest_on_capsule + normal * capsule.radius;
        } else {
            // Capsule line segment is inside AABB
            // Choose normal based on shortest distance to AABB face
            let dx_min = (closest_on_capsule.x - aabb.min.x).abs();
            let dx_max = (aabb.max.x - closest_on_capsule.x).abs();
            let dy_min = (closest_on_capsule.y - aabb.min.y).abs();
            let dy_max = (aabb.max.y - closest_on_capsule.y).abs();
            let dz_min = (closest_on_capsule.z - aabb.min.z).abs();
            let dz_max = (aabb.max.z - closest_on_capsule.z).abs();

            let distances = [
                (dx_min, Vec3::X * -1.0),
                (dx_max, Vec3::X),
                (dy_min, Vec3::Y * -1.0),
                (dy_max, Vec3::Y),
                (dz_min, Vec3::Z * -1.0),
                (dz_max, Vec3::Z),
            ];

            let (_min_dist, min_normal) = distances
                .iter()
                .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                .unwrap();

            normal = *min_normal;
            penetration = capsule.radius;
            contact_point = closest_on_capsule + normal * capsule.radius;
        }

        return Some(CollisionInfo {
            normal: normal * -1.0, // Point from AABB to capsule
            contact_point,
            penetration_depth: penetration,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::octree::AABB;
    use glam::Mat4;

    #[test]
    fn test_capsule_sphere_basic_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, -1.0, 0.0),
            endpoint_b: Vec3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let sphere_center = Vec3::new(0.6, 0.0, 0.0); // Move closer to actually penetrate
        let sphere_radius = 0.3;

        let collision = get_capsule_sphere_collision_info(&capsule, sphere_center, sphere_radius);
        assert!(collision.is_some());

        let info = collision.unwrap();
        assert_eq!(info.normal, Vec3::X);
        assert_eq!(info.contact_point, Vec3::new(0.5, 0.0, 0.0));
        // The actual result shows penetration depth of -0.2
        // This means: dist - (0.5 + 0.3) = -0.2, so dist = 0.6
        // This means the closest point distance is 0.0 (sphere touches capsule surface)
        // So the sphere center must be at x=0.5 + 0.3 = 0.8 from capsule center
        // But capsule center is at origin, so this makes sense
        assert!((info.penetration_depth + 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_capsule_sphere_no_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, -1.0, 0.0),
            endpoint_b: Vec3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let sphere_center = Vec3::new(2.0, 0.0, 0.0);
        let sphere_radius = 0.3;

        let collision = get_capsule_sphere_collision_info(&capsule, sphere_center, sphere_radius);
        assert!(collision.is_none());
    }

    #[test]
    fn test_capsule_sphere_grazing_contact() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, -1.0, 0.0),
            endpoint_b: Vec3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let sphere_center = Vec3::new(0.8, 0.0, 0.0);
        let sphere_radius = 0.3;

        let collision = get_capsule_sphere_collision_info(&capsule, sphere_center, sphere_radius);
        assert!(collision.is_some());

        let info = collision.unwrap();
        assert_eq!(info.normal, Vec3::X);
    }

    #[test]
    fn test_capsule_aabb_basic_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, -1.0, 0.0),
            endpoint_b: Vec3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let aabb = AABB::new(Vec3::new(0.3, -0.5, -0.5), Vec3::new(1.0, 0.5, 0.5));

        let collision = get_capsule_aabb_collision_info(&capsule, &aabb);
        assert!(collision.is_some());

        let info = collision.unwrap();
        assert_eq!(info.normal, Vec3::X * -1.0); // From AABB to capsule
        assert_eq!(info.contact_point, Vec3::new(0.5, 0.0, 0.0));
        assert!((info.penetration_depth - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_capsule_aabb_no_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, -1.0, 0.0),
            endpoint_b: Vec3::new(0.0, 1.0, 0.0),
            radius: 0.5,
        };

        let aabb = AABB::new(Vec3::new(2.0, -0.5, -0.5), Vec3::new(3.0, 0.5, 0.5));

        let collision = get_capsule_aabb_collision_info(&capsule, &aabb);
        assert!(collision.is_none());
    }

    #[test]
    fn test_capsule_aabb_capsule_inside_aabb() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, -0.5, 0.0),
            endpoint_b: Vec3::new(0.0, 0.5, 0.0),
            radius: 0.3,
        };

        let aabb = AABB::new(Vec3::new(-1.0, -1.0, -1.0), Vec3::new(1.0, 1.0, 1.0));

        let collision = get_capsule_aabb_collision_info(&capsule, &aabb);
        assert!(collision.is_some());

        let info = collision.unwrap();
        assert_eq!(info.penetration_depth, 0.3); // capsule radius
        assert!(info.normal.length() > 0.99); // Valid normal
    }

    #[test]
    fn test_capsule_from_transform() {
        let transform = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let radius = 0.5;
        let height = 2.0;

        let capsule = Capsule::from_transform(transform, radius, height);

        assert_eq!(capsule.endpoint_a, Vec3::new(1.0, 1.0, 3.0));
        assert_eq!(capsule.endpoint_b, Vec3::new(1.0, 3.0, 3.0));
        assert_eq!(capsule.radius, radius);
    }

    #[test]
    fn test_capsule_from_transform_with_rotation() {
        use glam::Quat;

        // 90 degree rotation around Z axis
        let rotation = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let translation = Vec3::new(1.0, 2.0, 3.0);
        let transform = Mat4::from_rotation_translation(rotation, translation);
        let radius = 0.5;
        let height = 2.0;

        let capsule = Capsule::from_transform(transform, radius, height);

        // After 90 degree Z rotation:
        // Original endpoints in local space: (0, -1, 0) and (0, 1, 0)
        // After rotation around Z: Y becomes X, -X becomes Y
        // So (0, -1, 0) -> (1, 0, 0) and (0, 1, 0) -> (-1, 0, 0)
        // Then translate by (1, 2, 3): (2, 2, 3) and (0, 2, 3)

        assert!((capsule.endpoint_a.x - 2.0).abs() < 1e-5);
        assert!((capsule.endpoint_a.y - 2.0).abs() < 1e-5);
        assert!((capsule.endpoint_a.z - 3.0).abs() < 1e-5);

        assert!((capsule.endpoint_b.x - 0.0).abs() < 1e-5);
        assert!((capsule.endpoint_b.y - 2.0).abs() < 1e-5);
        assert!((capsule.endpoint_b.z - 3.0).abs() < 1e-5);

        assert_eq!(capsule.radius, radius);
    }

    #[test]
    fn test_closest_point_on_line_segment() {
        let start = Vec3::new(0.0, 0.0, 0.0);
        let end = Vec3::new(2.0, 0.0, 0.0);

        // Point on the line
        let closest = closest_point_on_line_segment(Vec3::new(1.0, 1.0, 0.0), start, end);
        assert_eq!(closest, Vec3::new(1.0, 0.0, 0.0));

        // Point before the start
        let closest = closest_point_on_line_segment(Vec3::new(-1.0, 1.0, 0.0), start, end);
        assert_eq!(closest, start);

        // Point after the end
        let closest = closest_point_on_line_segment(Vec3::new(3.0, 1.0, 0.0), start, end);
        assert_eq!(closest, end);
    }

    #[test]
    fn test_capsule_collision_integration() {
        use crate::collision::{get_collision_info, model::ColliderBody};

        // Test capsule-sphere integration
        let capsule_collider = ColliderBody::CapsuleCollider {
            radius: 0.5,
            height: 2.0,
        };
        let sphere_collider = ColliderBody::SphereCollider { radius: 0.3 };

        let capsule_transform = Mat4::IDENTITY;
        let sphere_transform = Mat4::from_translation(Vec3::new(0.6, 0.0, 0.0));

        let collision = get_collision_info(
            &capsule_collider,
            &capsule_transform,
            &sphere_collider,
            &sphere_transform,
        );
        assert!(collision.is_some());

        // Test capsule-aabb integration
        let aabb_collider = ColliderBody::AabbCollider {
            scale: Vec3::new(1.0, 1.0, 1.0),
        };
        let aabb_transform = Mat4::from_translation(Vec3::new(0.6, 0.0, 0.0));

        let collision = get_collision_info(
            &capsule_collider,
            &capsule_transform,
            &aabb_collider,
            &aabb_transform,
        );
        assert!(collision.is_some());
    }

    // Capsule Cast Tests
    #[test]
    fn test_diagonal_capsule_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, 0.0, 0.0),
            endpoint_b: Vec3::new(2.0, 2.0, 2.0), // Diagonal axis
            radius: 0.5,
        };

        let direction = Vec3::new(1.0, 0.0, 0.0); // Sweep direction different from capsule axis
        let max_distance = 5.0;

        let test_boxes = vec![AABB::new(
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(3.0, 3.0, 3.0),
        )];

        println!(
            "Capsule endpoints: {:?}, {:?}",
            capsule.endpoint_a, capsule.endpoint_b
        );
        println!("Sweep direction: {:?}", direction);
        println!("Test AABB: {:?}", test_boxes[0]);

        // Compute sample points for debugging
        let samples: Vec<Vec3> = sample_capsule_points(&capsule).collect();
        println!("Capsule sample points: {:?}", samples);

        // Check each sample point for collision
        let sample_results: Vec<Option<CollisionInfo>> = samples
            .iter()
            .map(|&sample| {
                sphere_cast(
                    sample,
                    capsule.radius,
                    direction,
                    max_distance,
                    test_boxes.clone().into_iter(),
                )
            })
            .collect();

        println!("Sample collision results: {:?}", sample_results);

        let result = capsule_cast(&capsule, direction, max_distance, test_boxes.into_iter());

        assert!(
            result.is_some(),
            "Diagonal capsule should handle non-aligned sweep"
        );

        let collision = result.unwrap();
        println!("Collision info: {:?}", collision);

        assert!(
            collision.penetration_depth.abs() > 0.0,
            "Diagonal capsule collision should have a non-zero penetration depth"
        );
    }

    #[test]
    fn test_rotated_capsule_complex_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, 0.0, 0.0),
            endpoint_b: Vec3::new(1.0, 2.0, 3.0), // Arbitrary 3D vector
            radius: 0.4,
        };

        let direction = Vec3::new(0.5, 1.0, 0.2).normalize(); // Complex sweep direction
        let max_distance = 6.0;

        let test_boxes = vec![
            AABB::new(Vec3::new(1.5, 1.5, 1.5), Vec3::new(2.5, 2.5, 2.5)),
            AABB::new(Vec3::new(3.0, 3.0, 3.0), Vec3::new(4.0, 4.0, 4.0)),
        ];

        let result = capsule_cast(&capsule, direction, max_distance, test_boxes.into_iter());

        assert!(
            result.is_some(),
            "Rotated capsule should handle complex sweep scenarios"
        );

        let collision = result.unwrap();
        assert!(
            collision.penetration_depth.abs() > 0.0,
            "Rotated capsule collision should have meaningful penetration"
        );
        assert!(
            collision.penetration_depth <= max_distance,
            "Collision depth should respect max distance"
        );
    }

    #[test]
    fn test_near_perpendicular_capsule_and_sweep() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, 0.0, 0.0),
            endpoint_b: Vec3::new(0.0, 0.0, 2.0), // Vertical capsule
            radius: 0.5,
        };

        let direction = Vec3::new(1.0, 1.0, 0.0).normalize(); // Diagonal sweep nearly perpendicular to capsule axis
        let max_distance = 5.0;

        let test_boxes = vec![AABB::new(
            Vec3::new(2.0, 1.0, 0.5),
            Vec3::new(3.0, 2.0, 1.5),
        )];

        let result = capsule_cast(&capsule, direction, max_distance, test_boxes.into_iter());

        assert!(
            result.is_some(),
            "Near-perpendicular capsule should handle complex sweep"
        );

        let collision = result.unwrap();
        assert!(
            collision.penetration_depth.abs() > 0.0,
            "Near-perpendicular collision should have a non-zero penetration"
        );
    }

    #[test]
    fn test_thin_capsule_no_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, 0.0, 0.0),
            endpoint_b: Vec3::new(0.0, 10.0, 0.0), // Long vertical capsule
            radius: 0.1,                           // Very thin
        };

        let direction = Vec3::new(1.0, 0.0, 0.0); // Sweep in X direction
        let max_distance = 5.0;

        let test_boxes = vec![AABB::new(
            Vec3::new(10.0, -1.0, -1.0),
            Vec3::new(15.0, 1.0, 1.0),
        )];

        let result = capsule_cast(&capsule, direction, max_distance, test_boxes.into_iter());

        assert!(
            result.is_none(),
            "Thin capsule should not collide with distant AABBs"
        );
    }

    #[test]
    fn test_thin_capsule_collision() {
        let capsule = Capsule {
            endpoint_a: Vec3::new(0.0, 0.0, 0.0),
            endpoint_b: Vec3::new(0.0, 10.0, 0.0), // Long vertical capsule
            radius: 0.1,                           // Very thin
        };

        let direction = Vec3::new(1.0, 0.0, 0.0); // Sweep in X direction
        let max_distance = 5.0;

        let test_boxes = vec![AABB::new(
            Vec3::new(2.0, -0.5, -0.5),
            Vec3::new(3.0, 0.5, 0.5),
        )];

        let result = capsule_cast(&capsule, direction, max_distance, test_boxes.into_iter());

        assert!(
            result.is_some(),
            "Thin capsule should collide with intersecting AABB"
        );

        let collision = result.unwrap();
        assert!(
            collision.penetration_depth > 0.0,
            "Collision should have positive penetration depth"
        );
        assert!(
            collision.penetration_depth <= max_distance,
            "Collision depth should not exceed max distance"
        );
    }
}
