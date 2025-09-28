use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;

pub struct Camera {
    pub position: Vec3,
    rotation: Quat,
    pub speed: f32,
    pub sensitivity: f32,

    pitch: f32,
    yaw: f32,

    input_velocity: Vec3,
}

/// Extract pitch and yaw from a quaternion, assuming no roll (Z rotation).
/// Returns (pitch, yaw) in radians.
fn quaternion_to_pitch_yaw(q: Quat) -> (f32, f32) {
    // Forward vector from the quaternion
    let forward = q.mul_vec3(glam::Vec3::Z);

    // Yaw: angle around Y axis
    let yaw = forward.z.atan2(forward.x); // +Z is forward, so atan2(z, x)

    // Pitch: angle up/down
    let horizontal_length = (forward.x * forward.x + forward.z * forward.z).sqrt();
    let pitch = -forward.y.atan2(horizontal_length);

    (pitch, yaw)
}

impl Camera {
    pub fn new() -> Camera {
        let camera_position = Vec3::new(0.0, 1.0, 9.0);
        let pitch = 0.0;
        let yaw = 0.0;
        let camera_rotation = Quat::IDENTITY;
        Self {
            input_velocity: Vec3::ZERO,
            pitch,
            yaw,
            position: camera_position,
            rotation: camera_rotation,
            speed: 50.0,
            sensitivity: 0.01,
        }
    }

    pub fn tick(&mut self, dt: f32) {
        let requested_movement = self.input_velocity * dt;
        self.position += requested_movement;
        // Lock vertical position to not fall below ground plane
        self.position.y = self.position.y.max(1.0);
        self.input_velocity = Vec3::ZERO;
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
        let (pitch, yaw) = quaternion_to_pitch_yaw(rotation);
        self.pitch = pitch;
        self.yaw = yaw;
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

        self.update_rot_from_euler();
    }

    fn update_rot_from_euler(&mut self) {
        // Reconstruct the rotation from yaw and pitch
        let yaw_rotation = Quat::from_rotation_y(self.yaw);
        let pitch_rotation = Quat::from_rotation_x(self.pitch);

        // Combine yaw then pitch (Y * X)
        self.rotation = yaw_rotation * pitch_rotation;
    }

    pub fn process_keyboard(&mut self, key: KeyCode) {
        let camera_z_direction = self.rotation * Vec3::Z;
        let right = Vec3::Y.cross(camera_z_direction).normalize();

        match key {
            KeyCode::KeyW => self.input_velocity -= camera_z_direction * self.speed,
            KeyCode::KeyS => self.input_velocity += camera_z_direction * self.speed,
            KeyCode::KeyA => self.input_velocity -= right * self.speed,
            KeyCode::KeyD => self.input_velocity += right * self.speed,
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

    pub fn get_projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(45f32.to_radians(), 800.0 / 600.0, 0.1, 1000.0)
    }
}
