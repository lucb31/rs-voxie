use glam::IVec3;
use std::fmt::Debug;

use super::{bbs::IAabb, iter_commons::get_child_origin};

#[derive(Debug)]
pub struct OctreeNode<T> {
    pub(super) data: Option<T>,
    pub(super) children: Option<[Box<OctreeNode<T>>; 8]>,
}

impl<T> OctreeNode<T> {
    pub fn new() -> Self {
        Self {
            data: None,
            children: None,
        }
    }

    pub(super) fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    pub(super) fn default_children(&self) -> [Box<OctreeNode<T>>; 8] {
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
    pub(super) fn insert(&mut self, x: i32, y: i32, z: i32, size: usize, data: T) {
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
    pub(super) fn get(&mut self, x: i32, y: i32, z: i32, size: usize) -> Option<T> {
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
    pub(super) fn query_region_traverse(
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

    pub(super) fn traverse_depth_first(&self, res: &mut Vec<T>) {
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
