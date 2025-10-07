use glam::{Mat4, Quat, Vec3};

pub struct Camera {
    pub position: Vec3,
    rotation: Quat,
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
        Mat4::perspective_rh_gl(45f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0)
    }
}

pub trait CameraController {
    fn tick(&mut self, dt: f32, camera: &mut Camera, target_transform: &Mat4);
}
