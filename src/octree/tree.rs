use glam::IVec3;
use log::info;
use std::fmt::Debug;

use super::{IAabb, OctreeNodeIterator, iter_empty::OctreeEmptyNodeIterator, node::OctreeNode};

pub struct Octree<T> {
    // The current root node. If world needs to grow, we create a new root node and assign
    // the current to the minimum (index 0) of the new octant
    pub(super) root: OctreeNode<T>,
    // Size of the current root node
    // We only need to keep track of the root node. Internally the size will then be halfed
    // during recursive indexing
    pub(super) size: usize,
    // Position of origin (minimum corner) of root node in TREE space
    pub(super) origin: IVec3,
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
    pub fn iter_region(&self, region_tree_space: IAabb) -> OctreeNodeIterator<T> {
        OctreeNodeIterator::new(region_tree_space, self)
    }

    pub fn iter_empty_within_region(
        &self,
        region_tree_space: IAabb,
    ) -> impl Iterator<Item = IVec3> {
        OctreeEmptyNodeIterator::new(region_tree_space, self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use glam::IVec3;

    use crate::octree::{IAabb, Octree, iter_commons::get_child_origin};

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
        let result = root.iter_region(test_region).collect::<Vec<&TestData>>();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].a, 1);
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

        let it = root.iter_region(test_region);
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
            tree.iter_region(IAabb::new(&IVec3::new(0, 0, 0), 1))
                .collect::<Vec<&TestData>>()
                .len(),
            1
        );
        tree.grow(16);
        assert_eq!(tree.get_size(), 4);
        assert_eq!(
            tree.iter_region(IAabb::new(&IVec3::new(0, 0, 0), 1))
                .collect::<Vec<&TestData>>()
                .len(),
            1
        );
    }
}
