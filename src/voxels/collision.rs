use glam::{Mat4, Vec3, Vec4Swizzles};
use hecs::World;

use crate::{
    collision::{
        ColliderBody, CollisionEvent, CollisionInfo,
        capsule::{Capsule, get_capsule_aabb_collision_info},
        get_sphere_aabb_collision_info,
    },
    octree::{AABB, IAabb},
    systems::physics::Transform,
};

use super::{Voxel, VoxelWorld};

/// Tag component. Only entities that have both a ColliderBody and this tag component
/// will be check for collision with the voxel world
pub struct VoxelCollider;

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
    let iter = world.iter_region_voxels(sphere_box_region_i);
    iter.filter_map(move |voxel| {
        let vox_collider = voxel.get_collider()?;
        get_sphere_aabb_collision_info(&center, radius, &vox_collider)
    })
}

pub fn coarse_collision_voxel_world_capsule(
    world: &VoxelWorld,
    transform: Mat4,
    radius: f32,
    height: f32,
) -> impl Iterator<Item = Voxel> {
    let center = transform.w_axis.xyz();
    debug_assert!(center.is_finite());
    debug_assert!(radius > 0.00001);
    debug_assert!(height > 0.00001);

    // Coarse-grained BB test
    let coarse_bb_f = AABB::new_center(&center, height + radius);
    let coarse_bb_i = IAabb::from(&coarse_bb_f);
    world.iter_region_voxels(coarse_bb_i)
}

fn iter_capsule_collision(
    world: &VoxelWorld,
    transform: Mat4,
    radius: f32,
    height: f32,
) -> impl Iterator<Item = CollisionInfo> {
    // Coarse-grained BB test
    let iter = coarse_collision_voxel_world_capsule(world, transform, radius, height);

    // Fine-grained collision test
    let capsule = Capsule::from_transform(transform, radius, height);
    iter.filter_map(move |voxel| {
        let vox_collider = voxel.get_collider()?;
        get_capsule_aabb_collision_info(&capsule, &vox_collider)
    })
}

pub fn system_voxel_world_collisions(
    world: &mut World,
    voxel_world: &VoxelWorld,
) -> Vec<CollisionEvent> {
    let mut all_collisions: Vec<CollisionEvent> = Vec::new();
    for (_entity, (transform, collider)) in world
        .query::<(&Transform, &ColliderBody)>()
        .with::<&VoxelCollider>()
        .iter()
    {
        match collider {
            ColliderBody::SphereCollider { radius } => {
                let center = transform.0.w_axis.xyz();
                all_collisions.extend(iter_sphere_collision(voxel_world, center, *radius).map(
                    |info| CollisionEvent {
                        info,
                        a: _entity,
                        b: None,
                    },
                ));
            }
            ColliderBody::AabbCollider { .. } => todo!("AABB voxel collision not implemented"),
            ColliderBody::CapsuleCollider { radius, height } => {
                all_collisions.extend(
                    iter_capsule_collision(voxel_world, transform.0, *radius, *height).map(
                        |info| CollisionEvent {
                            info,
                            a: _entity,
                            b: None,
                        },
                    ),
                );
            }
        };
    }
    all_collisions
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::{
        collision::CollisionInfo,
        voxels::{VoxelWorld, collision::iter_sphere_collision},
    };

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
}
