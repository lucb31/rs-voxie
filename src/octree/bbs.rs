use glam::{IVec3, Vec3};

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

    #[cfg(test)]
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

#[derive(Clone, Debug, PartialEq)]
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
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
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

    pub fn _area(&self) -> i32 {
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

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::octree::bbs::AABB;

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
