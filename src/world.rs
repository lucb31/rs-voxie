use noise::{NoiseFn, Perlin};
use std::{cell::RefCell, error::Error, rc::Rc};

use glam::Vec3;

use crate::{
    octree::{AABB, WorldTree},
    voxel::Voxel,
};

fn count_voxel_neighbors(voxel: &Voxel, world: &WorldTree<Rc<RefCell<Voxel>>>) -> i32 {
    let origin = voxel.position;
    let bb = AABB::new(&origin, 2.0);
    let close_voxels = world.query_region(&bb);
    let mut neighbors = 0;
    for voxel in &close_voxels {
        let close_voxel = voxel.borrow();
        // Skip myself
        if close_voxel.position == origin {
            continue;
        }
        // Must be off by max 1 in all axis
        let dist = origin - close_voxel.position;
        if dist.x.abs() > 1.1 {
            continue;
        }
        if dist.y.abs() > 1.1 {
            continue;
        }
        if dist.z.abs() > 1.1 {
            continue;
        }
        let candidate_bb = close_voxel.get_bb();
        if bb.intersects(&candidate_bb) {
            neighbors += 1;
        }
        debug_assert!(
            neighbors < 27,
            "Too many neighbors ({neighbors}) at {origin:?}"
        );
    }
    neighbors
}

// Used for testing purposes
pub fn generate_cubic_world(initial_size: usize) -> WorldTree<Rc<RefCell<Voxel>>> {
    println!("Generating cubic world size {initial_size}");
    let mut world = WorldTree::new(initial_size, Vec3::ZERO);
    let mut nodes = 0;
    let half = initial_size as i32 / 2;
    for x in -half + 1..half + 1 {
        for z in -half + 1..half + 1 {
            for y in -half + 1..half + 1 {
                let mut voxel = Voxel::new();
                voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                world.insert(voxel.position, Rc::new(RefCell::new(voxel)));
                nodes += 1;
            }
        }
    }
    println!("World generation produced {nodes} nodes");
    update_voxel_visibility(&world).expect("Could not determine invis nodes");
    world
}

// NOTE: Required until rendering becomes smarter
// Currently if we allow for same height as width and depth, we just
// generate a bunch of cubes, that are not visible and should not be drawn
// once rendering is smarter
const HEIGHT_LIMIT: i32 = 32;

pub fn generate_world(
    initial_size: usize,
) -> Result<WorldTree<Rc<RefCell<Voxel>>>, Box<dyn Error>> {
    let mut world = WorldTree::new(initial_size, Vec3::ZERO);
    println!("Generating world size {initial_size}");
    // TUNING
    const SEED: u32 = 99;
    let scale = 0.03;
    let perlin = Perlin::new(SEED);

    let mut nodes = 0;
    let half = initial_size as i32 / 2;
    let max_height = HEIGHT_LIMIT.min(half - 1) as f64;
    for x in -half + 1..half {
        let fx = x as f64 * scale;
        for z in -half + 1..half {
            let fz = z as f64 * scale;
            let noise_val = perlin.get([fx, fz]);
            let max_y = ((noise_val + 1.0) * (max_height / 2.0)).floor() as i32;
            // NOTE: As long as there is no way to 'dig down' into the world,
            // there is no point filling up the world below the surface voxels.
            // Once that is added we need to sample all 3d points or generate on the fly
            // -3 is to add SOME depth, otherwise there will be gaps in 'staircase' shapes
            for y in max_y - 3..max_y {
                let mut voxel = Voxel::new();
                voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                world.insert(voxel.position, Rc::new(RefCell::new(voxel)));
                nodes += 1;
            }
        }
    }
    println!("World generation produced {nodes} nodes");
    update_voxel_visibility(&world).expect("Could not determine invis nodes");
    Ok(world)
}

fn update_voxel_visibility(tree: &WorldTree<Rc<RefCell<Voxel>>>) -> Result<(), Box<dyn Error>> {
    let all_voxels = tree.get_all_depth_first();
    let mut invis_nodes = 0;
    for voxel in &all_voxels {
        let neighbors = count_voxel_neighbors(&voxel.borrow(), tree);
        let mut node = voxel.borrow_mut();
        if neighbors >= 26 {
            node.visible = false;
            invis_nodes += 1;
        } else {
            node.visible = true;
        }
    }
    println!(
        "{} / {} world voxels visible",
        all_voxels.len() - invis_nodes,
        all_voxels.len()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::{voxel::Voxel, world::generate_cubic_world};

    #[test]
    fn test_size_2_visiblity() {
        let world = generate_cubic_world(2);
        let all_nodes = world.get_all_depth_first();
        assert_eq!(all_nodes.len(), 8);
        // In a 2x2x2 cube all cubes are visible edges
        let visible_cubes: Vec<Rc<RefCell<Voxel>>> = all_nodes
            .iter()
            .filter(|x| x.borrow().visible)
            .cloned()
            .collect();
        assert_eq!(visible_cubes.len(), 8);
    }

    #[test]
    fn test_size_4_visiblity() {
        let world = generate_cubic_world(4);
        let all_nodes = world.get_all_depth_first();
        assert_eq!(all_nodes.len(), 64);
        // 4x4 a Front & back face
        // 2x4 a Left & right face
        // 2x2 a Top & bototm face
        let visible_cubes: Vec<Rc<RefCell<Voxel>>> = all_nodes
            .iter()
            .filter(|x| x.borrow().visible)
            .cloned()
            .collect();
        assert_eq!(visible_cubes.len(), 4 * 4 * 2 + 2 * 4 * 2 + 2 * 2 * 2);
    }
    #[test]
    fn test_size_8_visiblity() {
        let world = generate_cubic_world(8);
        let all_nodes = world.get_all_depth_first();
        assert_eq!(all_nodes.len(), 512);
        let visible_cubes: Vec<Rc<RefCell<Voxel>>> = all_nodes
            .iter()
            .filter(|x| x.borrow().visible)
            .cloned()
            .collect();
        // 8x8 a Front & back face
        // 6x8 a Left & right face
        // 6x6 a Top & bototm face
        assert_eq!(visible_cubes.len(), 8 * 8 * 2 + 6 * 8 * 2 + 6 * 6 * 2);
    }
}
