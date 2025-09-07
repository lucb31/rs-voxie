use glam::{Mat4, Vec3};

pub struct Camera {
    transform: Mat4,
}

impl Camera {
    pub fn new() -> Camera {
        Self {
            transform: Mat4::from_translation(Vec3::new(0.0, 0.0, 3.0)),
        }
    }

    pub fn get_view_projection_matrix(&self) -> Mat4 {
        self.get_projection_matrix() * self.get_view_matrix()
    }

    pub fn translate(&mut self, translation: Vec3) {
        let transform = Mat4::from_translation(translation);
        self.transform = self.transform.mul_mat4(&transform);
    }

    fn get_view_matrix(&self) -> Mat4 {
        self.transform.inverse()
    }

    fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(45f32.to_radians(), 800.0 / 600.0, 0.1, 100.0)
    }
}
