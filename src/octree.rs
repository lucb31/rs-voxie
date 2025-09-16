#[derive(Debug)]
struct OctreeNode<T> {
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
    T: Clone,
{
    pub fn insert(&mut self, x: usize, y: usize, z: usize, size: usize, data: T) {
        debug_assert!(x < size);
        debug_assert!(y < size);
        debug_assert!(z < size);
        if size == 1 {
            self.data = Some(data);
            return;
        }

        let half = size / 2;
        let index = get_child_index(x, y, z, half);
        if self.children.is_none() {
            self.children = Some(self.default_children());
        }
        let child = self.children.as_mut().unwrap();
        child[index].insert(x % half, y % half, z % half, half, data);
    }

    pub fn get(&mut self, x: usize, y: usize, z: usize, size: usize) -> Option<T> {
        if size == 1 {
            return self.data.clone();
        }
        let half = size / 2;
        let index = get_child_index(x, y, z, half);
        if self.children.is_none() {
            return None.clone();
        }
        let children = self.children.as_mut().unwrap();
        children[index].get(x, y, z, half).clone()
    }

    pub fn get_all_depth_first(&self, size: usize) -> Vec<T> {
        // NOTE: Might be capacity overkill. This is the maximum size we'll ever need
        let mut res: Vec<T> = Vec::with_capacity(size * size * size);
        self.traverse_depth_first(&mut res);
        res
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
fn get_child_index(x: usize, y: usize, z: usize, half: usize) -> usize {
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

struct WorldTree<T> {
    // The current root node. If world needs to grow, we create a new root node and assign
    // the current to the center (child index 7) of the new octant
    root: OctreeNode<T>,
    // Size of the current root node
    // We only need to keep track of the root node. Internally the size will then be halfed
    // during recursive indexing
    size: usize,
    // We need to know where (0,0,0) is in the tree
    origin_offset: (i32, i32, i32), // offset of root's min corner
}

impl<T> WorldTree<T> {
    pub fn grow(&mut self) {
        // Need to always grow by one power of 2
        let new_size = self.size * 2;
        let mut new_root: OctreeNode<T> = OctreeNode::new();
        let target_index = 7;
    }
}

#[cfg(test)]
mod tests {
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
        let mut root: OctreeNode<TestData> = OctreeNode::new();
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
                    root.insert(x, y, z, size, my_data);
                    nodes_inserted += 1;
                }
            }
        }
        println!("Total nodes inserted: {nodes_inserted}");

        let data_vec = root.get_all_depth_first(size);

        assert_eq!(data_vec.len(), nodes_inserted);
    }
}
