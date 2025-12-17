use glam::IVec3;

use super::node::OctreeNode;

pub(super) struct StackItem<'a, T> {
    pub(super) node: &'a OctreeNode<T>,
    pub(super) origin: IVec3,
    pub(super) size: usize,
}

pub(super) fn get_child_origin(parent_origin: &IVec3, size: usize, index: usize) -> IVec3 {
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
