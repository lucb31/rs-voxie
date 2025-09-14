use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vec3,
    rotation: Quat,
    pub speed: f32,
    pub sensitivity: f32,

    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new() -> Camera {
        let camera_position = Vec3::new(0.0, 1.0, 9.0);
        let pitch = 0.0;
        let yaw = 0.0;
        let yaw_rotation = Quat::from_rotation_y(yaw);
        let pitch_rotation = Quat::from_rotation_x(pitch);
        let camera_rotation = yaw_rotation * pitch_rotation;
        Self {
            pitch,
            yaw,
            position: camera_position,
            rotation: camera_rotation,
            speed: 5000.0,
            sensitivity: 0.01,
        }
    }

    pub fn process_mouse_movement(&mut self, dx: f64, dy: f64) {
        let dx = dx as f32;
        let dy = dy as f32;

        // Update yaw and pitch
        self.yaw -= dx * self.sensitivity;
        self.pitch -= dy * self.sensitivity;

        // Clamp pitch to [-89°, 89°] to prevent flipping
        let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01; // ~89.4°
        self.pitch = self.pitch.clamp(-pitch_limit, pitch_limit);

        // Reconstruct the rotation from yaw and pitch
        let yaw_rotation = Quat::from_rotation_y(self.yaw);
        let pitch_rotation = Quat::from_rotation_x(self.pitch);

        // Combine yaw then pitch (Y * X)
        self.rotation = yaw_rotation * pitch_rotation;
    }

    pub fn process_keyboard(&mut self, key: KeyCode, dt: f32) {
        let camera_z_direction = self.rotation * Vec3::Z;
        let right = Vec3::Y.cross(camera_z_direction).normalize();

        let mut updated_position = self.position;
        match key {
            KeyCode::KeyW => updated_position -= camera_z_direction * self.speed * dt,
            KeyCode::KeyS => updated_position += camera_z_direction * self.speed * dt,
            KeyCode::KeyA => updated_position -= right * self.speed * dt,
            KeyCode::KeyD => updated_position += right * self.speed * dt,
            _ => {}
        }

        // Lock vertical position to not vall below ground plane
        self.position = Vec3::new(
            updated_position.x,
            updated_position.y.max(1.0),
            updated_position.z,
        );
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
