use std::fmt::Debug;

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
