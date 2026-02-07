use glam::{Mat4, Quat, Vec3};

pub struct BlenderOrbitCamera {
    /// World-space pivot (orbit center)
    pub pivot: Vec3,

    /// Distance from pivot (dolly)
    pub distance: f32,

    /// Accumulated yaw (around world Y)
    yaw: f32,

    /// Accumulated pitch (around camera-local X)
    pitch: f32,

    /// Radians per pixel
    pub sensitivity: f32,

    /// Pitch clamp to avoid flipping (± ~89°)
    pub max_pitch: f32,
}

impl BlenderOrbitCamera {
    pub fn new(pivot: Vec3, distance: f32) -> Self {
        Self {
            pivot,
            distance,
            yaw: 0.0,
            pitch: 0.0,
            sensitivity: 0.005,
            max_pitch: 1.553343, // ~89 degrees
        }
    }

    /// Middle mouse drag (orbit)
    pub fn orbit(&mut self, mouse_dx: f32, mouse_dy: f32) {
        self.yaw -= mouse_dx * self.sensitivity;
        self.pitch -= mouse_dy * self.sensitivity;

        self.pitch = self.pitch.clamp(-self.max_pitch, self.max_pitch);
    }

    /// Scroll wheel (dolly)
    pub fn dolly(&mut self, delta: f32) {
        self.distance = (self.distance + delta).clamp(0.1, 1000.0);
    }

    /// Shift + MMB (pan)
    pub fn pan(&mut self, dx: f32, dy: f32) {
        let orientation = self.orientation();
        let right = orientation * Vec3::X;
        let up = orientation * Vec3::Y;

        let pan_speed = self.distance * 0.001;
        self.pivot += (-right * dx + up * dy) * pan_speed;
    }

    /// Camera orientation from yaw/pitch (stable)
    fn orientation(&self) -> Quat {
        let yaw_q = Quat::from_rotation_y(self.yaw);
        let pitch_q = Quat::from_rotation_x(self.pitch);
        yaw_q * pitch_q
    }

    /// Camera world transform (camera → world)
    pub fn camera_transform(&self) -> Mat4 {
        let orientation = self.orientation();
        let forward = orientation * Vec3::Z;
        let position = self.pivot + forward * self.distance;

        Mat4::from_rotation_translation(orientation, position)
    }

    /// View matrix (world → camera)
    pub fn view_matrix(&self) -> Mat4 {
        self.camera_transform().inverse()
    }
}
