use glam::{Mat4, Quat, Vec3};

use crate::octree::IAabb;

pub struct Camera {
    pub position: Vec3,
    rotation: Quat,
}

#[derive(Debug)]
pub struct Plane {
    pub normal: glam::Vec3,
    pub d: f32, // plane equation: normal·x + d = 0
}

#[derive(Debug)]
pub struct Frustum {
    pub planes: [Plane; 6],
}

impl Frustum {
    pub fn contains_aabb(&self, aabb: &IAabb) -> bool {
        for plane in &self.planes {
            // Select the vertex farthest in the direction of the plane normal
            let p = glam::vec3(
                if plane.normal.x >= 0.0 {
                    aabb.max.x as f32
                } else {
                    aabb.min.x as f32
                },
                if plane.normal.y >= 0.0 {
                    aabb.max.y as f32
                } else {
                    aabb.min.y as f32
                },
                if plane.normal.z >= 0.0 {
                    aabb.max.z as f32
                } else {
                    aabb.min.z as f32
                },
            );

            // If this point is outside the plane, the entire AABB is outside
            if plane.normal.dot(p) + plane.d < 0.0 {
                return false;
            }
        }
        true
    }
}

impl Camera {
    pub fn new() -> Camera {
        let camera_position = Vec3::ZERO;
        let camera_rotation = Quat::IDENTITY;
        Self {
            position: camera_position,
            rotation: camera_rotation,
        }
    }

    pub fn set_rotation(&mut self, rot: Quat) {
        self.rotation = rot;
    }

    pub fn get_rotation(&self) -> Quat {
        self.rotation
    }

    pub fn look_at(&mut self, target_position: Vec3) {
        let view_matrix = Mat4::look_at_rh(self.position, target_position, Vec3::Y);
        let transform = view_matrix.inverse();
        self.position = transform.w_axis.truncate();
        let rotation = Quat::from_mat4(&transform);
        self.rotation = rotation;
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    // NOTE: Equal to inverse of camera transform
    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position).inverse()
    }

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(60f32.to_radians(), 1920.0 / 1080.0, 0.1, 1000.0)
    }

    // Extract planes from the combined view-projection matrix
    pub fn get_frustum(&self) -> Frustum {
        // Helper to extract a plane from combinations of rows
        fn make_plane(a: [f32; 4], b: [f32; 4]) -> Plane {
            let n = glam::vec3(a[0] + b[0], a[1] + b[1], a[2] + b[2]);
            let d = a[3] + b[3];
            // Normalize
            let inv_len = 1.0 / n.length();
            Plane {
                normal: n * inv_len,
                d: d * inv_len,
            }
        }

        let vp = self.get_view_projection_matrix();
        let rows = vp.transpose().to_cols_array_2d();
        let r0 = rows[0];
        let r1 = rows[1];
        let r2 = rows[2];
        let r3 = rows[3];

        Frustum {
            planes: [
                make_plane(r3, r0),                               // left
                make_plane(r3, [-r0[0], -r0[1], -r0[2], -r0[3]]), // right
                make_plane(r3, r1),                               // bottom
                make_plane(r3, [-r1[0], -r1[1], -r1[2], -r1[3]]), // top
                make_plane(r3, r2),                               // near
                make_plane(r3, [-r2[0], -r2[1], -r2[2], -r2[3]]), // far
            ],
        }
    }
}

pub trait CameraController {
    fn tick(&mut self, dt: f32, camera: &mut Camera, target_transform: &Mat4);
}
