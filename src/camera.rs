use glam::{Mat4, Vec3};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new() -> Camera {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
        }
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        let sensitivity = 0.002;
        self.yaw += (dx as f32) * sensitivity;
        self.pitch -= (dy as f32) * sensitivity;
        self.pitch = self.pitch.clamp(-1.54, 1.54); // prevent flip
    }

    pub fn process_keyboard(&mut self, key: KeyCode, dt: f32) {
        let dir = Vec3::new(self.yaw.cos(), 0.0, self.yaw.sin()).normalize();
        let right = -Vec3::Y.cross(dir).normalize();
        let speed = 500.0;

        match key {
            KeyCode::KeyW => self.position += dir * speed * dt,
            KeyCode::KeyS => self.position -= dir * speed * dt,
            KeyCode::KeyA => self.position -= right * speed * dt,
            KeyCode::KeyD => self.position += right * speed * dt,
            _ => {}
        }
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    fn get_view_matrix(&self) -> Mat4 {
        let dir = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        let target = self.position + dir;
        Mat4::look_at_rh(self.position, target, Vec3::Y)
    }

    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(45f32.to_radians(), 800.0 / 600.0, 0.1, 100.0)
    }
}
