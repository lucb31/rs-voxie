use std::fmt::Debug;

use glam::Vec3;

#[derive(Debug)]
pub struct OctreeNode<T> {
    data: Option<T>,
    children: Option<[Box<OctreeNode<T>>; 8]>,
}

impl<T> OctreeNode<T> {
    pub fn new() -> Self {
        Self {
            data: None,
            children: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    fn default_children(&self) -> [Box<OctreeNode<T>>; 8] {
        [
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
            Box::new(OctreeNode::new()),
        ]
    }
}

#[derive(Debug)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(origin: &Vec3, size: f32) -> AABB {
        debug_assert!(size > 0.0, "Size of BB needs to be > 0");
        let half = size / 2.0;
        Self {
            min: Vec3::new(origin.x - half, origin.y - half, origin.z - half),
            max: Vec3::new(origin.x + half, origin.y + half, origin.z + half),
        }
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        // For each axis, check if one box is completely to one side of the other
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn contains(&self, other: &AABB) -> bool {
        self.min.x <= other.min.x
            && self.min.y <= other.min.y
            && self.min.z <= other.min.z
            && self.max.x >= other.max.x
            && self.max.y >= other.max.y
            && self.max.z >= other.max.z
    }
}

impl<T> OctreeNode<T>
where
    T: Clone + Debug,
{
    pub fn insert(&mut self, x: i32, y: i32, z: i32, size: usize, data: T) {
        // Assertions: By design decision we do not allow for negative
        // coordinate values. Negative positions in world space are represented
        // via the origin_offset
        debug_assert!(x < size as i32);
        debug_assert!(x >= 0);
        debug_assert!(y < size as i32);
        debug_assert!(y >= 0);
        debug_assert!(z < size as i32);
        debug_assert!(z >= 0);

        // Exit condition
        if size == 1 {
            self.data = Some(data);
            return;
        }

        // Recursion
        let half = (size / 2) as i32;
        let index = get_child_index(x, y, z, half);
        if self.children.is_none() {
            self.children = Some(self.default_children());
        }
        let child = self.children.as_mut().unwrap();
        child[index].insert(x % half, y % half, z % half, half as usize, data);
    }

    pub fn get(&mut self, x: i32, y: i32, z: i32, size: usize) -> Option<T> {
        if size == 1 {
            return self.data.clone();
        }
        let half = size / 2;
        let index = get_child_index(x, y, z, half as i32);
        if self.children.is_none() {
            return None.clone();
        }
        let children = self.children.as_mut().unwrap();
        children[index].get(x, y, z, half).clone()
    }

    fn query_region_traverse(&self, size: usize, origin: &Vec3, region: &AABB, res: &mut Vec<T>) {
        let current_boundary = AABB::new(origin, size as f32);
        // Check if boundary intersects with current node boundary
        // Exit cond: If it does not intersect at all
        // We dont want to add any data or traverse any further
        if !current_boundary.intersects(region) {
            return;
        }
        // Hit a leave. Finally some data. Dont need to traverse further
        if self.is_leaf() {
            if let Some(data) = self.data.clone() {
                res.push(data);
            }
            return;
        }

        // Recursion
        for (index, child) in self.children.as_ref().unwrap().iter().enumerate() {
            let child_origin = get_child_origin(origin, size, index);
            child.query_region_traverse(size / 2, &child_origin, region, res);
        }
    }

    fn traverse_depth_first(&self, res: &mut Vec<T>) {
        // Exit case
        if self.is_leaf() {
            if let Some(data) = self.data.clone() {
                res.push(data);
            }
            return;
        }

        // Recursion
        for child in self.children.as_ref().unwrap() {
            child.traverse_depth_first(res);
        }
    }
}

// Figures out in which octant to place a coordinate
fn get_child_index(x: i32, y: i32, z: i32, half: i32) -> usize {
    let mut index = 0;
    if x >= half {
        index |= 1;
    }
    if y >= half {
        index |= 2;
    }
    if z >= half {
        index |= 4;
    }
    index
}

fn get_child_origin(parent_origin: &Vec3, size: usize, index: usize) -> Vec3 {
    let quarter = size as f32 / 4.0;

    let x = if index & 1 != 0 {
        parent_origin.x + quarter
    } else {
        parent_origin.x - quarter
    };
    let y = if index & 2 != 0 {
        parent_origin.y + quarter
    } else {
        parent_origin.y - quarter
    };
    let z = if index & 4 != 0 {
        parent_origin.z + quarter
    } else {
        parent_origin.z - quarter
    };
    Vec3::new(x, y, z)
}

pub struct WorldTree<T> {
    // The current root node. If world needs to grow, we create a new root node and assign
    // the current to the center (child index 7) of the new octant
    root: OctreeNode<T>,
    // Size of the current root node
    // We only need to keep track of the root node. Internally the size will then be halfed
    // during recursive indexing
    size: usize,
    // We need to know where (0,0,0) is in the tree
    origin_offset: Vec3, // offset of root's min corner
}

impl<T> WorldTree<T>
where
    T: Clone + Debug,
{
    pub fn new(size: usize, origin: Vec3) -> Self {
        let root: OctreeNode<T> = OctreeNode::new();
        Self {
            root,
            size,
            origin_offset: origin,
        }
    }

    pub fn insert(&mut self, pos: Vec3, data: T) {
        let (x, y, z) = transform_i32_pos_to_u32_triple(pos, self.origin_offset, self.size);
        // TODO: Once confirmed this works, instead of typecasting, let OctreeNode work with
        // unsigned ints only
        self.root
            .insert(x as i32, y as i32, z as i32, self.size, data);
    }

    pub fn get_all_depth_first(&self) -> Vec<T> {
        // NOTE: Might be capacity overkill. This is the maximum size we'll ever need
        let mut res: Vec<T> = Vec::with_capacity(self.size * self.size * self.size);
        self.root.traverse_depth_first(&mut res);
        res
    }

    //pub fn get() { // Reminder to offset by world center offset when transforming positions in
    // world space into octree space}

    pub fn query_region(&self, region: &AABB) -> Vec<T> {
        let mut res: Vec<T> = vec![];
        self.root
            .query_region_traverse(self.size, &self.origin_offset, region, &mut res);
        res
    }
}

fn transform_i32_pos_to_u32_triple(pos: Vec3, offset: Vec3, size: usize) -> (u32, u32, u32) {
    let fsize = size as f32;
    let offset_corrected_pos = pos - offset + Vec3::ONE * (fsize / 2.0 - 1.0);
    let x = offset_corrected_pos.x;
    let y = offset_corrected_pos.y;
    let z = offset_corrected_pos.z;
    debug_assert!(
        x >= 0.0 && x < fsize,
        "x-Coordinate out of bounds: Calculated to {x}, Received position {pos}"
    );
    debug_assert!(
        y >= 0.0 && y < fsize,
        "y-Coordinate out of bounds: Calculated to {y}, Received position {pos}"
    );
    debug_assert!(z >= 0.0 && z < fsize, "z-Coordinate out of bounds: {z}");
    (x as u32, y as u32, z as u32)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use glam::Vec3;

    use crate::octree::WorldTree;
    use crate::octree::get_child_origin;
    use crate::octree::transform_i32_pos_to_u32_triple;

    use super::AABB;
    use super::IVec3;
    use super::OctreeNode;

    #[derive(Clone, Debug)]
    struct TestData {
        a: i32,
        b: bool,
    }

    #[test]
    fn test_get_child_origin() {
        let parent_origin = Vec3::ZERO;
        let size = 8;
        let mut set: HashSet<IVec3> = HashSet::new();
        for i in 0..8 {
            let child_origin = get_child_origin(&parent_origin, size, i);
            assert_eq!(
                (child_origin.x - parent_origin.x).abs(),
                size as f32 / 4.0,
                "Incorrect x-distance between child origin {child_origin:?} and parent origin {parent_origin:?}"
            );
            assert_eq!(
                (child_origin.y - parent_origin.y).abs(),
                size as f32 / 4.0,
                "Incorrect y-distance between child origin {child_origin:?} and parent origin {parent_origin:?}"
            );
            assert_eq!(
                (child_origin.z - parent_origin.z).abs(),
                size as f32 / 4.0,
                "Incorrect z-distance between child origin {child_origin:?} and parent origin {parent_origin:?}"
            );
            // NOTE: We're just converting to IVec here because that implements the required hash
            // traits
            set.insert(IVec3::new(
                child_origin.x as i32,
                child_origin.y as i32,
                child_origin.z as i32,
            ));
        }
        assert_eq!(
            set.len(),
            8,
            "Did not return 8 unique vectors for 8 octants"
        );
    }

    #[test]
    fn test_i32_pos_transform() {
        // correct position by offset to ensure all positons are within interval [0, size[
        // Valid positions for offset 0 are ]-half;half]
        let pos = Vec3::ONE * 4.0;
        let offset = Vec3::ZERO;
        let size: usize = 8;
        let (x, y, z) = transform_i32_pos_to_u32_triple(pos, offset, size);
        assert_eq!(x, 7);
        assert_eq!(y, 7);
        assert_eq!(z, 7);

        let pos = Vec3::ONE * -3.0;
        let (x, y, z) = transform_i32_pos_to_u32_triple(pos, offset, size);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(z, 0);

        let pos = Vec3::ONE * -2.0;
        let offset = Vec3::ONE;
        let (x, y, z) = transform_i32_pos_to_u32_triple(pos, offset, size);
        assert_eq!(x, 0);
        assert_eq!(y, 0);
        assert_eq!(z, 0);

        let pos = Vec3::ONE * -2.0;
        let offset = -Vec3::ONE;
        let (x, y, z) = transform_i32_pos_to_u32_triple(pos, offset, size);
        assert_eq!(x, 2);
        assert_eq!(y, 2);
        assert_eq!(z, 2);
    }

    #[test]
    #[should_panic]
    fn test_i32_pos_transform_fails_out_of_bounds() {
        let pos = Vec3::ONE * -4.0;
        let offset = Vec3::ZERO;
        let size: usize = 8;
        let (_, _, _) = transform_i32_pos_to_u32_triple(pos, offset, size);
    }

    #[test]
    fn test_add_and_get_node() {
        let size: usize = 8;
        let mut root: OctreeNode<TestData> = OctreeNode::new();
        assert!(
            root.is_leaf(),
            "Root is not considered leaf node although no children were added yet"
        );
        let my_data = TestData { a: 3, b: false };
        root.insert(0, 0, 0, size, my_data);
        assert!(
            !root.is_leaf(),
            "Root is considered leaf, althoug a child was added"
        );
        let stored_data = root.get(0, 0, 0, size);
        assert!(stored_data.is_some());
        let data = stored_data.unwrap();
        assert_eq!(data.a, 3);
        assert_eq!(data.b, false);

        let my_data = TestData { a: 3, b: false };
        root.insert(0, 0, 0, 8, my_data);
    }

    #[test]
    #[should_panic(expected = "assertion failed: x < size")]
    fn test_add_node_outside_bounds() {
        let size: usize = 8;
        let mut root: OctreeNode<TestData> = OctreeNode::new();
        let my_data = TestData { a: 3, b: false };
        root.insert(8, 0, 0, size, my_data);
        let option = root.get(8, 0, 0, size);
        assert!(
            option.is_none(),
            "Retrieval of data outside of bounds returned a result"
        );
    }

    #[test]
    fn test_iterating_tree_depth_first() {
        let size: i32 = 8;
        let mut root: WorldTree<TestData> = WorldTree::new(size as usize, Vec3::ZERO);
        let mut nodes_inserted = 0;
        let half = size / 2;
        for x in 0..size {
            // Adding some conditions to make the tree more sparse
            if x % 2 == 0 {
                continue;
            }
            for y in 0..size {
                // Adding some conditions to make the tree more sparse
                if y % 3 == 0 {
                    continue;
                }
                for z in 0..size {
                    // Adding some conditions to make the tree more sparse
                    if z % 4 == 0 {
                        continue;
                    }
                    let my_data = TestData {
                        a: x * y * z,
                        b: false,
                    };
                    // Since we specified origin at ZERO, we need to map points
                    // to [-half;half] interval
                    let pos = Vec3::new(x as f32, y as f32, z as f32) - half as f32 * Vec3::ONE;
                    root.insert(pos, my_data);
                    nodes_inserted += 1;
                }
            }
        }
        println!("Total nodes inserted: {nodes_inserted}");

        let data_vec = root.get_all_depth_first();

        assert_eq!(
            data_vec.len(),
            nodes_inserted,
            "Less items contained in tree than added by loop"
        );
    }

    #[test]
    fn test_intersection_true() {
        let a = AABB::new(&Vec3::ZERO, 1.0);
        let b = AABB::new(&Vec3::ONE, 1.0);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_intersection_close_but_false() {
        let a = AABB::new(&Vec3::ZERO, 1.0);
        let b = AABB::new(&Vec3::ONE, 0.9);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_intersection_false() {
        let a = AABB::new(&Vec3::ZERO, 1.0);
        let b = AABB::new(&(Vec3::ONE * 2.0), 1.0);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_simple_region_query() {
        // Assemble 2x2x2 tree position at tree origin (0,0,0)
        let size: usize = 2;
        let mut root: WorldTree<TestData> = WorldTree::new(size, Vec3::ZERO);
        let my_data = TestData { a: 3, b: false };
        // Insert a test node at origin
        root.insert(Vec3::ZERO, my_data.clone());

        // HIT region around origin with very small size
        let region = AABB::new(&Vec3::ZERO, 0.01);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 1);
        // HIT region around origin with size 1
        let region = AABB::new(&Vec3::ZERO, 1.0);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 1);
        // HIT region around .5,.5,.5 with size 1
        let region = AABB::new(&(Vec3::ONE * 0.5), 1.0);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 1);
        // HIT region around 1,1,1 with size 2
        let region = AABB::new(&Vec3::ONE, 2.0);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 1);
        // MISS Test region around 1,1,1 with size 1
        let region = AABB::new(&Vec3::ONE, 1.0);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 0);
        // MISS Test region around 2,1,1 with size 1
        let region = AABB::new(&Vec3::new(2.0, 1.0, 1.0), 1.0);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_region_query_one_node_at_region_origin() {
        // Assemble tree with node at 3,2,0
        let size: usize = 8;
        let mut root: WorldTree<TestData> = WorldTree::new(size, Vec3::ZERO);
        let my_data = TestData { a: 3, b: false };
        let node_position = Vec3::new(3.0, 2.0, 0.0);
        root.insert(node_position, my_data.clone());

        // HIT region at node position 3,2,0
        let region = AABB::new(&node_position, 1.0);
        let results = root.query_region(&region);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_region_query() {
        let size: usize = 8;
        let mut root: WorldTree<TestData> = WorldTree::new(size, Vec3::ZERO);
        let my_data = TestData { a: 3, b: false };
        root.insert(Vec3::new(2.0, 2.0, 0.0), my_data.clone());
        root.insert(Vec3::new(3.0, 2.0, 0.0), my_data.clone());
        root.insert(Vec3::new(-3.0, 2.0, 0.0), my_data.clone());

        let region = AABB::new(&Vec3::new(3.0, 2.0, 0.0), 2.0);
        let results = root.query_region(&region);

        assert_eq!(results.len(), 2);
    }
}
