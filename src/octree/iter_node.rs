use super::{
    IAabb, Octree,
    iter_commons::{StackItem, get_child_origin},
};
use std::fmt::Debug;

pub struct OctreeNodeIterator<'a, T> {
    stack: Vec<StackItem<'a, T>>,
    region: IAabb,
}

impl<'a, T> OctreeNodeIterator<'a, T> {
    pub(super) fn new(region_tree_space: IAabb, octree: &Octree<T>) -> OctreeNodeIterator<T> {
        OctreeNodeIterator {
            region: region_tree_space,
            stack: vec![StackItem {
                node: &octree.root,
                origin: octree.origin,
                size: octree.size,
            }],
        }
    }
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
