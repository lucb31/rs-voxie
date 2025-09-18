use glam::IVec3;

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
    min: IVec3,
    max: IVec3,
}

impl AABB {
    pub fn new(origin: &IVec3, size: usize) -> AABB {
        let half = (size / 2) as i32;
        Self {
            min: IVec3::new(origin.x - half, origin.y - half, origin.z - half),
            max: IVec3::new(origin.x + half, origin.y + half, origin.z + half),
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
    T: Clone,
{
    pub fn insert(&mut self, x: i32, y: i32, z: i32, size: usize, data: T) {
        debug_assert!(x < size as i32);
        debug_assert!(y < size as i32);
        debug_assert!(z < size as i32);
        if size == 1 {
            self.data = Some(data);
            return;
        }

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

    fn query_region_traverse(&self, size: usize, origin: &IVec3, region: &AABB, res: &mut Vec<T>) {
        // Hit a leave. Finally some data. Dont need to traverse further
        if self.is_leaf() {
            if let Some(data) = self.data.clone() {
                res.push(data);
            }
            return;
        }
        let current_boundary = AABB::new(origin, size);
        // Check if boundary intersects with current node boundary
        // Exit cond: If it does not intersect at all
        // We dont want to add any data or traverse any further
        if !current_boundary.intersects(region) {
            return;
        }

        // Recursion
        // WARNING: origin might not be ok to pass through
        for (index, child) in self.children.as_ref().unwrap().iter().enumerate() {
            let child_origin = get_child_origin(origin, size, index);
            child.query_region_traverse(size / 2, &child_origin, region, res);
        }
    }

    // Private recursion call
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

pub struct WorldTree<T> {
    // The current root node. If world needs to grow, we create a new root node and assign
    // the current to the center (child index 7) of the new octant
    root: OctreeNode<T>,
    // Size of the current root node
    // We only need to keep track of the root node. Internally the size will then be halfed
    // during recursive indexing
    size: usize,
    // We need to know where (0,0,0) is in the tree
    origin_offset: IVec3, // offset of root's min corner
}

impl<T> WorldTree<T>
where
    T: Clone,
{
    pub fn new(size: usize) -> Self {
        let root: OctreeNode<T> = OctreeNode::new();
        Self {
            root,
            size,
            origin_offset: IVec3::new(0, 0, 0),
        }
    }

    pub fn insert(&mut self, x: i32, y: i32, z: i32, data: T) {
        self.root.insert(x, y, z, self.size, data);
    }

    pub fn get_all_depth_first(&self) -> Vec<T> {
        // NOTE: Might be capacity overkill. This is the maximum size we'll ever need
        let mut res: Vec<T> = Vec::with_capacity(self.size * self.size * self.size);
        self.root.traverse_depth_first(&mut res);
        res
    }

    pub fn query_region(&self, region: &AABB) -> Vec<T> {
        let mut res: Vec<T> = vec![];
        self.root
            .query_region_traverse(self.size, &self.origin_offset, region, &mut res);
        res
    }

    pub fn get(&mut self, x: i32, y: i32, z: i32) -> Option<T> {
        self.root.get(x, y, z, self.size)
    }

    pub fn grow(&mut self) {
        todo!("Implementation missing");
        // Need to always grow by one power of 2
        let new_size = self.size * 2;
        // let mut new_root: OctreeNode<T> = OctreeNode::new();
        let target_index = 7;
    }
}

#[cfg(test)]
mod tests {
    use crate::octree::WorldTree;

    use super::AABB;
    use super::IVec3;
    use super::OctreeNode;

    #[derive(Clone, Debug)]
    struct TestData {
        a: i32,
        b: bool,
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
        let size: usize = 8;
        let mut root: WorldTree<TestData> = WorldTree::new(size);
        let mut nodes_inserted = 0;
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
                        a: (x * y * z) as i32,
                        b: false,
                    };
                    root.insert(x as i32, y as i32, z as i32, my_data);
                    nodes_inserted += 1;
                }
            }
        }
        println!("Total nodes inserted: {nodes_inserted}");

        let data_vec = root.get_all_depth_first();

        assert_eq!(data_vec.len(), nodes_inserted);
    }

    #[test]
    fn test_intersection_true() {
        let a = AABB {
            min: IVec3::new(0, 0, 0),
            max: IVec3::new(2, 2, 2),
        };
        let b = AABB {
            min: IVec3::new(1, 1, 1),
            max: IVec3::new(3, 3, 3),
        };
        assert!(a.intersects(&b));
    }

    #[test]
    fn test_intersection_false() {
        let a = AABB {
            min: IVec3::new(0, 0, 0),
            max: IVec3::new(1, 1, 1),
        };
        let b = AABB {
            min: IVec3::new(2, 2, 2),
            max: IVec3::new(3, 3, 3),
        };
        assert!(!a.intersects(&b));
    }

    #[test]
    fn test_region_query() {
        let size: usize = 8;
        let mut root: WorldTree<TestData> = WorldTree::new(size);
        let my_data = TestData { a: 3, b: false };
        root.insert(2, 2, 0, my_data.clone());
        root.insert(3, 2, 0, my_data.clone());
        root.insert(7, 2, 0, my_data.clone());

        let region = AABB::new(&IVec3::new(3, 2, 0), 2);
        let results = root.query_region(&region);

        assert_eq!(results.len(), 2);
    }
}
