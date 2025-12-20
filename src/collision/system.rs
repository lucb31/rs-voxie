use hecs::{Entity, World};

use crate::{
    collision::{ColliderBody, CollisionEvent, get_collision_info},
    systems::physics::Transform,
};

pub fn system_collisions(world: &mut World) -> Vec<CollisionEvent> {
    let mut all_collisions: Vec<CollisionEvent> = Vec::new();

    let mut query = world.query::<(&Transform, &ColliderBody)>();
    let colliders: Vec<(Entity, (&Transform, &ColliderBody))> = query.iter().collect();

    // Iterate over all unique pairs
    for i in 0..colliders.len() {
        for j in (i + 1)..colliders.len() {
            let (entity_a, (transform_a, collider_a)) = colliders[i];
            let (entity_b, (transform_b, collider_b)) = colliders[j];

            // TODO: Collision mask mechanism is missing. We're checking & catching a lot of collision events,
            // we're probably not interested in tracking

            let collision_info =
                get_collision_info(collider_a, &transform_a.0, collider_b, &transform_b.0);
            if let Some(info) = collision_info {
                all_collisions.push(CollisionEvent {
                    info,
                    a: entity_a,
                    b: Some(entity_b),
                });
            }
        }
    }
    all_collisions
}
