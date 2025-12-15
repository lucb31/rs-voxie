use glam::IVec3;
use log::info;
use std::fmt::Debug;

use super::bbs::IAabb;

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

impl<T> OctreeNode<T>
where
    T: Clone + Debug,
{
    // These x,y,z coordinates are local to the current node
    pub fn insert(&mut self, x: i32, y: i32, z: i32, size: usize, data: T) {
        debug_assert!(x < size as i32, "x val {x} is too high for size {size}");
        debug_assert!(y < size as i32, "y val {y} is too high for size {size}");
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

    #[cfg(test)]
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

    /// # Arguments
    /// * `origin` - Minimum corner of the current octree in octree space
    /// * @deprecate once iterator concept proven
    fn query_region_traverse(
        &self,
        size: usize,
        origin: &IVec3,
        region: &IAabb,
        res: &mut Vec<T>,
        uninitialized: &mut Option<Vec<IVec3>>,
    ) {
        let current_boundary = IAabb::new(origin, size);
        // Exit condition: Region does not intersect
        // If boundary does not intersect with current node boundary,
        // we dont want to add any data or traverse any further
        if !current_boundary.intersects(region) {
            return;
        }
        // Exit condition: Hit a leaf
        if self.is_leaf() {
            if self.data.is_some() {
                let data = self.data.clone().unwrap();
                res.push(data);
            } else if let Some(uninit) = uninitialized.as_mut() {
                uninit.push(*origin);
            }
            return;
        }
        // Recursion
        for (index, child) in self.children.as_ref().unwrap().iter().enumerate() {
            let child_origin = get_child_origin(origin, size, index);
            child.query_region_traverse(size / 2, &child_origin, region, res, uninitialized);
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

pub struct QueryResult<T> {
    pub data: Vec<T>,
    pub uninitialized: Vec<IVec3>,
}

pub struct Octree<T> {
    // The current root node. If world needs to grow, we create a new root node and assign
    // the current to the minimum (index 0) of the new octant
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

    /// Query the given region in **octree** space
    pub fn query_region(&self, region_octree_space: &IAabb) -> QueryResult<T> {
        let mut res: Vec<T> = vec![];
        let mut uninitialized = Some(vec![]);
        self.root.query_region_traverse(
            self.size,
            &self.origin,
            region_octree_space,
            &mut res,
            &mut uninitialized,
        );
        QueryResult {
            data: res,
            uninitialized: uninitialized.unwrap(),
        }
    }

    pub fn get_total_region_world_space(&self, chunk_size: usize) -> IAabb {
        IAabb::new(&self.origin, self.size * chunk_size)
    }

    pub fn grow(&mut self, chunk_size: usize) {
        let mut new_root: OctreeNode<T> = OctreeNode::new();
        let mut children = new_root.default_children();
        let old_root = std::mem::replace(&mut self.root, OctreeNode::new());
        let old_size = self.size;
        children[0] = Box::new(old_root);
        new_root.children = Some(children);
        self.root = new_root;
        self.size *= 2;
        info!(
            "Grew tree from size {} to size {}. Now covering region {:?}",
            old_size,
            self.size,
            self.get_total_region_world_space(chunk_size),
        );
    }

    /// Returns iterator within region in **octree_space**
    pub fn iter_region(&self, region: &IAabb) -> OctreeNodeIterator<T> {
        OctreeNodeIterator {
            region: region.clone(),
            stack: vec![StackItem {
                node: &self.root,
                origin: self.origin,
                size: self.size,
            }],
        }
    }
}

struct StackItem<'a, T> {
    node: &'a OctreeNode<T>,
    origin: IVec3,
    size: usize,
}

pub struct OctreeNodeIterator<'a, T> {
    stack: Vec<StackItem<'a, T>>,
    region: IAabb,
}

impl<'a, T> Iterator for OctreeNodeIterator<'a, T>
where
    T: Clone + Debug,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.pop() {
            let current_boundary = IAabb::new(&item.origin, item.size);
            if !current_boundary.intersects(&self.region) {
                continue;
            }
            // Recursion push all children to the stack
            let node = item.node;
            if node.is_leaf() {
                if let Some(data) = node.data.as_ref() {
                    return Some(data);
                }
            } else {
                for (index, child) in node.children.as_ref().unwrap().iter().enumerate() {
                    let child_origin = get_child_origin(&item.origin, item.size, index);
                    self.stack.push(StackItem {
                        node: child.as_ref(),
                        origin: child_origin,
                        size: item.size / 2,
                    });
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use glam::IVec3;

    use crate::octree::{IAabb, Octree, node::get_child_origin};

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
    fn test_region_query() {
        let size: usize = 4;
        let mut root = Octree::new(size);
        root.insert(IVec3::new(0, 0, 0), TestData { a: 1, b: false });
        root.insert(IVec3::new(2, 0, 0), TestData { a: 2, b: false });
        root.insert(IVec3::new(0, 2, 0), TestData { a: 3, b: false });
        root.insert(IVec3::new(1, 1, 2), TestData { a: 4, b: false });
        let test_region = IAabb::new(&IVec3::ZERO, 2);
        let result = root.query_region(&test_region);
        assert_eq!(result.data.len(), 1);
        assert_eq!(result.data[0].a, 1);
    }

    #[test]
    fn test_region_iterator() {
        let size: usize = 4;
        let mut root = Octree::new(size);
        root.insert(IVec3::new(0, 0, 0), TestData { a: 1, b: false });
        root.insert(IVec3::new(2, 0, 0), TestData { a: 2, b: false });
        root.insert(IVec3::new(0, 2, 0), TestData { a: 3, b: false });
        root.insert(IVec3::new(1, 1, 2), TestData { a: 4, b: false });
        let test_region = IAabb::new(&IVec3::ZERO, 2);

        let it = root.iter_region(&test_region);
        let result: Vec<&TestData> = it.collect();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].a, 1);
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
    fn test_octree_grow() {
        let mut tree: Octree<TestData> = Octree::new(2);
        let my_data = TestData { a: 3, b: false };
        tree.insert(IVec3::new(0, 0, 0), my_data);
        assert_eq!(tree.get_size(), 2);
        assert_eq!(
            tree.query_region(&IAabb::new(&IVec3::new(0, 0, 0), 1))
                .data
                .len(),
            1
        );
        tree.grow(16);
        assert_eq!(tree.get_size(), 4);
        assert_eq!(
            tree.query_region(&IAabb::new(&IVec3::new(0, 0, 0), 1))
                .data
                .len(),
            1
        );
    }
}
