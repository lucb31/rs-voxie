use glam::{Mat4, Vec3, Vec4Swizzles};

use crate::octree::AABB;

use super::{
    CollisionInfo, get_sphere_aabb_collision_info, get_sphere_sphere_collision_info,
    model::ColliderBody,
};

fn is_axis_aligned(t: &Mat4) -> bool {
    let (_scale, rotation, _translation) = Mat4::to_scale_rotation_translation(t);
    rotation.is_near_identity()
}

pub fn get_collision_info(
    collider_a: &ColliderBody,
    transform_a: &Mat4,
    collider_b: &ColliderBody,
    transform_b: &Mat4,
) -> Option<CollisionInfo> {
    let position_a = transform_a.w_axis.xyz();
    let position_b = transform_b.w_axis.xyz();
    match collider_a {
        ColliderBody::SphereCollider { radius: radius_a } => match collider_b {
            ColliderBody::SphereCollider { radius: radius_b } => {
                get_sphere_sphere_collision_info(position_a, *radius_a, position_b, *radius_b)
            }
            ColliderBody::AabbCollider { scale: scale_b } => {
                debug_assert!(
                    is_axis_aligned(transform_b),
                    "Missing collision implementation: Object B has rotation. Don't know how to calculate non axis-aligned BB yet"
                );
                let aabb_b = AABB::from_center_and_scale(&position_b, scale_b);
                get_sphere_aabb_collision_info(&position_a, *radius_a, &aabb_b)
            }
        },
        ColliderBody::AabbCollider { scale: scale_a } => {
            debug_assert!(
                is_axis_aligned(transform_a),
                "Missing collision implementation: Object A has rotation. Don't know how to calculate non axis-aligned BB yet"
            );
            let aabb_a = AABB::from_center_and_scale(&position_a, scale_a);
            match collider_b {
                ColliderBody::SphereCollider { radius: radius_b } => {
                    get_sphere_aabb_collision_info(&position_b, *radius_b, &aabb_a)
                }
                ColliderBody::AabbCollider { scale: scale_b } => {
                    debug_assert!(
                        is_axis_aligned(transform_b),
                        "Missing collision implementation: Object B has rotation. Don't know how to calculate non axis-aligned BB yet"
                    );
                    let aabb_b = AABB::from_center_and_scale(&position_b, scale_b);
                    match aabb_a.intersects(&aabb_b) {
                        true => Some(CollisionInfo {
                            normal: Vec3::ZERO,
                            contact_point: position_a,
                            penetration_depth: 0.0,
                        }),
                        false => None,
                    }
                }
            }
        }
    }
}
