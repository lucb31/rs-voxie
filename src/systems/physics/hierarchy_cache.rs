use std::collections::HashMap;

use log::trace;

use super::{Parent, Transform};

pub struct HierarchyCache {
    depths: HashMap<hecs::Entity, usize>,
    pub entities_by_depth: Vec<(usize, hecs::Entity)>,
    pub is_dirty: bool,
}

impl HierarchyCache {
    pub fn new() -> HierarchyCache {
        Self {
            depths: HashMap::new(),
            entities_by_depth: Vec::new(),
            is_dirty: true,
        }
    }

    fn invalidate(&mut self) {
        self.is_dirty = true;
    }

    pub fn rebuild(&mut self, world: &hecs::World) {
        // Compute depths
        self.depths = compute_depths(world);

        // Collect entities sorted by depth
        let mut entities_by_depth: Vec<(usize, hecs::Entity)> = self
            .depths
            .iter()
            .map(|(entity, depth)| (*depth, *entity))
            .collect();
        entities_by_depth.sort_by_key(|(depth, _)| *depth);
        self.entities_by_depth = entities_by_depth;

        self.is_dirty = false;
    }
}

fn compute_depths(world: &hecs::World) -> HashMap<hecs::Entity, usize> {
    let mut depths: HashMap<hecs::Entity, usize> = HashMap::new();

    // First pass: identify root entities (no parent)
    for (entity, _) in world.query::<&Transform>().iter() {
        if world.get::<&Parent>(entity).is_err() {
            depths.insert(entity, 0);
        }
    }

    // Iteratively assign depths to children
    let mut changed = true;
    let mut iterations = 0;
    while changed {
        changed = false;
        for (entity, parent) in world.query::<&Parent>().iter() {
            if !depths.contains_key(&entity) {
                if let Some(&parent_depth) = depths.get(&parent.0) {
                    depths.insert(entity, parent_depth + 1);
                    changed = true;
                }
            }
        }
        iterations += 1;
    }
    trace!("Depth computation took {iterations} iterations");

    depths
}

/// Find all descendant entities of a parent entity.
///
/// Returns a vector of entity IDs that are descendants of `parent_entity`
/// (at any depth level). This includes direct children, grandchildren, and so on.
pub fn find_descendants<T: hecs::Query>(
    world: &hecs::World,
    parent_entity: hecs::Entity,
) -> Vec<hecs::Entity> {
    world
        .query::<T>()
        .iter()
        .filter_map(|(entity, _)| {
            if is_descendant_of(world, entity, parent_entity) {
                Some(entity)
            } else {
                None
            }
        })
        .collect()
}

/// Check if `entity` is a descendant of `ancestor` by walking the parent chain
fn is_descendant_of(world: &hecs::World, entity: hecs::Entity, ancestor: hecs::Entity) -> bool {
    let mut current = entity;
    while let Ok(parent) = world.get::<&Parent>(current) {
        if parent.0 == ancestor {
            return true;
        }
        current = parent.0;
    }
    false
}
