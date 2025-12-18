use glam::IVec3;

use super::{IAabb, Octree, iter_commons::StackItem, iter_commons::get_child_origin};
use std::fmt::Debug;

pub struct OctreeEmptyNodeIterator<'a, T> {
    stack: Vec<StackItem<'a, T>>,
    region: IAabb,
}

impl<'a, T> OctreeEmptyNodeIterator<'a, T> {
    pub(super) fn new(region_tree_space: IAabb, octree: &Octree<T>) -> OctreeEmptyNodeIterator<T> {
        OctreeEmptyNodeIterator {
            region: region_tree_space,
            stack: vec![StackItem {
                node: &octree.root,
                origin: octree.origin,
                size: octree.size,
            }],
        }
    }
}

impl<'a, T> Iterator for OctreeEmptyNodeIterator<'a, T>
where
    T: Clone + Debug,
{
    type Item = IVec3;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.stack.pop() {
            let current_boundary = IAabb::new(&item.origin, item.size);
            if !current_boundary.intersects(&self.region) {
                continue;
            }
            let node = item.node;
            if node.is_leaf() {
                if node.data.is_none() {
                    return Some(item.origin);
                }
            } else {
                // Recursion push all children to the stack
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
    use glam::IVec3;

    use crate::octree::{IAabb, Octree, iter_empty::OctreeEmptyNodeIterator, node::OctreeNode};

    #[test]
    fn empty_leafs_are_returned() {
        let mut tree: Octree<usize> = Octree::new(2);
        tree.insert(IVec3::ZERO, 0);
        let region = IAabb::new(&IVec3::ZERO, 8);

        let iter = OctreeEmptyNodeIterator::new(region, &tree);
        let result: Vec<_> = iter.collect();

        assert_eq!(result.len(), 7);
    }

    #[test]
    fn empty_leafs_are_returned_after_multiple_inserts() {
        let mut tree: Octree<usize> = Octree::new(2);
        tree.insert(IVec3::ZERO, 0);
        tree.insert(IVec3::ONE, 0);
        let region = IAabb::new(&IVec3::ZERO, 8);

        let iter = OctreeEmptyNodeIterator::new(region, &tree);
        let result: Vec<_> = iter.collect();

        assert_eq!(result.len(), 6);
    }

    #[test]
    fn outside_region_respected() {
        let mut tree: Octree<usize> = Octree::new(2);
        tree.insert(IVec3::ZERO, 0);
        tree.insert(IVec3::ONE, 0);
        let region = IAabb::new(&IVec3::splat(5), 8);

        let iter = OctreeEmptyNodeIterator::new(region, &tree);
        let result: Vec<_> = iter.collect();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn tree_size_4() {
        let mut tree: Octree<usize> = Octree::new(4);
        tree.insert(IVec3::ZERO, 0);
        let region = IAabb::new(&IVec3::ZERO, 8);

        let iter = OctreeEmptyNodeIterator::new(region, &tree);
        let result: Vec<_> = iter.collect();

        // size 4
        // Level 0 => 8 nodes, 1 of initialized with 8 children
        // Level 1 => First node has 8 children, one of them initialized with 8 children
        // Level 2 => First node has the data
        // => Total 8 + 8 + 1 = 17 nodes
        // - 3 initialized nodes (root with 8 children, data node father with 8 children, data node)
        // = 14
        assert_eq!(result.len(), 14);
    }
}
