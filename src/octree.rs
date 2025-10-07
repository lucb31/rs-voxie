use std::fmt::Debug;

use glam::{IVec3, Vec3};

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
    pub fn new(min: Vec3, max: Vec3) -> AABB {
        debug_assert!(max.x > min.x, "Invalid bounds: x axis");
        debug_assert!(max.y > min.y, "Invalid bounds: y axis");
        debug_assert!(max.z > min.z, "Invalid bounds: z axis");
        Self { min, max }
    }

    pub fn new_center(center: &Vec3, size: f32) -> AABB {
        debug_assert!(size > 0.0, "Size of BB needs to be > 0");
        let half = size / 2.0;
        Self {
            min: Vec3::new(center.x - half, center.y - half, center.z - half),
            max: Vec3::new(center.x + half, center.y + half, center.z + half),
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

#[derive(Debug)]
pub struct IAabb {
    pub min: IVec3,
    pub max: IVec3,
}
impl IAabb {
    pub fn new_rect(min: IVec3, max: IVec3) -> IAabb {
        debug_assert!(max.x > min.x, "Invalid bounds: x axis");
        debug_assert!(max.y > min.y, "Invalid bounds: y axis");
        debug_assert!(max.z > min.z, "Invalid bounds: z axis");
        Self { min, max }
    }

    // Returns Some(IAabb) if valid min and max values provided.
    // Returns none if area of BB would be <= 0
    fn try_new_rect(min: IVec3, max: IVec3) -> Option<IAabb> {
        if max.x <= min.x || max.y <= min.y || max.z <= min.z {
            return None;
        }
        Some(IAabb::new_rect(min, max))
    }

    pub fn new(min: &IVec3, size: usize) -> IAabb {
        debug_assert!(size > 0, "Size of BB needs to be > 0");
        let max = min + IVec3::ONE * size as i32;
        Self { min: *min, max }
    }

    pub fn intersects(&self, other: &IAabb) -> bool {
        // For each axis, check if one box is completely to one side of the other
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    pub fn intersection(&self, other: &IAabb) -> Option<IAabb> {
        let overlap_min = IVec3::new(
            self.min.x.max(other.min.x),
            self.min.y.max(other.min.y),
            self.min.z.max(other.min.z),
        );
        let overlap_max = IVec3::new(
            self.max.x.min(other.max.x),
            self.max.y.min(other.max.y),
            self.max.z.min(other.max.z),
        );
        IAabb::try_new_rect(overlap_min, overlap_max)
    }

    pub fn contains(&self, other: &IAabb) -> bool {
        self.min.x <= other.min.x
            && self.min.y <= other.min.y
            && self.min.z <= other.min.z
            && self.max.x >= other.max.x
            && self.max.y >= other.max.y
            && self.max.z >= other.max.z
    }

    pub fn area(&self) -> i32 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y) * (self.max.z - self.min.z)
    }
}
impl From<&AABB> for IAabb {
    fn from(other: &AABB) -> Self {
        IAabb::new_rect(
            IVec3::new(
                other.min.x.floor() as i32,
                other.min.y.floor() as i32,
                other.min.z.floor() as i32,
            ),
            IVec3::new(
                other.max.x.ceil() as i32,
                other.max.y.ceil() as i32,
                other.max.z.ceil() as i32,
            ),
        )
    }
}

impl<T> OctreeNode<T>
where
    T: Clone + Debug,
{
    // These x,y,z coordinates are local to the current node
    pub fn insert(&mut self, x: i32, y: i32, z: i32, size: usize, data: T) {
        debug_assert!(x < size as i32);
        debug_assert!(y < size as i32);
        debug_assert!(z < size as i32, "z val {z} is too high for size {size}");

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

    // These x,y,z coordinates are local to the current node
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

    // Origin is the minimum corner of the current octree in tree space
    fn query_region_traverse(&self, size: usize, origin: &IVec3, region: &IAabb, res: &mut Vec<T>) {
        let current_boundary = IAabb::new(origin, size);
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

fn get_child_origin(parent_origin: &IVec3, size: usize, index: usize) -> IVec3 {
    let half = (size / 2) as i32;

    let x = if index & 1 != 0 {
        parent_origin.x + half
    } else {
        parent_origin.x
    };
    let y = if index & 2 != 0 {
        parent_origin.y + half
    } else {
        parent_origin.y
    };
    let z = if index & 4 != 0 {
        parent_origin.z + half
    } else {
        parent_origin.z
    };
    IVec3::new(x, y, z)
}

pub struct Octree<T> {
    // The current root node. If world needs to grow, we create a new root node and assign
    // the current to the center (child index 7) of the new octant
    root: OctreeNode<T>,
    // Size of the current root node
    // We only need to keep track of the root node. Internally the size will then be halfed
    // during recursive indexing
    size: usize,
    // Position of origin (minimum corner) of root node in TREE space
    origin: IVec3,
}

impl<T> Octree<T>
where
    T: Clone + Debug,
{
    // Initialize new world with origin at (0,0,0) **tree** coordinates
    pub fn new(size: usize) -> Self {
        let root: OctreeNode<T> = OctreeNode::new();
        Self {
            root,
            size,
            origin: IVec3::ZERO,
        }
    }

    // Insert data into tree at tree space position
    pub fn insert(&mut self, pos_tree_space: IVec3, data: T) {
        // Check bounds
        debug_assert!(
            pos_tree_space.x >= self.origin.x,
            "X {} Out of bounds and we don't know how to grow yet.",
            pos_tree_space.x
        );
        debug_assert!(
            pos_tree_space.y >= self.origin.y,
            "y {} Out of bounds and we don't know how to grow yet.",
            pos_tree_space.y
        );
        debug_assert!(
            pos_tree_space.z >= self.origin.z,
            "z {} Out of bounds and we don't know how to grow yet. {}",
            pos_tree_space.z,
            self.origin
        );
        self.root.insert(
            pos_tree_space.x,
            pos_tree_space.y,
            pos_tree_space.z,
            self.size,
            data,
        );
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    pub fn get_all_depth_first(&self) -> Vec<T> {
        // NOTE: Might be capacity overkill. This is the maximum size we'll ever need
        let mut res: Vec<T> = Vec::with_capacity(self.size * self.size * self.size);
        self.root.traverse_depth_first(&mut res);
        res
    }

    // WARN: When querying, the region needs to be in octree space!
    pub fn query_region(&self, region: &IAabb) -> Vec<T> {
        let mut res: Vec<T> = vec![];
        self.root
            .query_region_traverse(self.size, &self.origin, region, &mut res);
        res
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use glam::IVec3;
    use glam::Vec3;

    use crate::octree::get_child_origin;

    use super::AABB;
    use super::OctreeNode;

    #[derive(Clone, Debug)]
    struct TestData {
        a: i32,
        b: bool,
    }

    #[test]
    fn test_get_child_origin() {
        let parent_origin = IVec3::ZERO;
        let size = 8;
        let mut set: HashSet<IVec3> = HashSet::new();
        for i in 0..8 {
            let child_origin = get_child_origin(&parent_origin, size, i);
            set.insert(child_origin);
        }
        assert_eq!(
            set.len(),
            8,
            "Did not return 8 unique vectors for 8 octants"
        );
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
    fn test_intersection_true() {
        let a = AABB::new_center(&Vec3::ZERO, 1.0);
        let b = AABB::new_center(&Vec3::ONE, 1.0);
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_intersection_close_but_false() {
        let a = AABB::new_center(&Vec3::ZERO, 1.0);
        let b = AABB::new_center(&Vec3::ONE, 0.9);
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_intersection_false() {
        let a = AABB::new_center(&Vec3::ZERO, 1.0);
        let b = AABB::new_center(&(Vec3::ONE * 2.0), 1.0);
        assert!(!a.intersects(&b));
    }
}
