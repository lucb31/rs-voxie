use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vec3,
    rotation: Quat,
    pub speed: f32,
    pub sensitivity: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::from_rotation_y(0.0),
            speed: 500.0,
            sensitivity: 0.002,
        }
    }

    pub fn process_mouse_movement(&mut self, dx: f64, dy: f64) {
        self.rotation *= Quat::from_rotation_y((-dx as f32) * self.sensitivity);
        self.rotation *= Quat::from_rotation_x((-dy as f32) * self.sensitivity);
    }

    pub fn process_keyboard(&mut self, key: KeyCode, dt: f32) {
        let camera_z_direction = self.rotation * Vec3::Z;
        let right = Vec3::Y.cross(camera_z_direction).normalize();

        match key {
            KeyCode::KeyW => self.position -= camera_z_direction * self.speed * dt,
            KeyCode::KeyS => self.position += camera_z_direction * self.speed * dt,
            KeyCode::KeyA => self.position -= right * self.speed * dt,
            KeyCode::KeyD => self.position += right * self.speed * dt,
            _ => {}
        }
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    // NOTE: Equal to inverse of camera transform
    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position).inverse()
    }

    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(45f32.to_radians(), 800.0 / 600.0, 0.1, 100.0)
    }
}
