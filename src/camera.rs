use glam::{Mat4, Quat, Vec3};

pub struct Camera {
    position: Vec3,
    rotation: Quat,
    velocity: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            velocity: Vec3::new(0.0, 0.0, 0.0),
        }
    }

    pub fn process(&mut self, dt: f32) {
        self.position += self.velocity * dt;
    }

    pub fn set_velocity(&mut self, vel: Vec3) {
        self.velocity = vel;
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    fn get_view_matrix(&self) -> Mat4 {
        let transform = Mat4::from_rotation_translation(self.rotation, self.position);
        transform.inverse()
    }

    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(45f32.to_radians(), 800.0 / 600.0, 0.1, 100.0)
    }
}
