use glam::{Mat3, Mat4, Quat};

use super::camera::{Camera, CameraController};

pub struct ThirdPersonCam {
    pub distance: f32,
}

impl ThirdPersonCam {
    pub fn new() -> ThirdPersonCam {
        Self { distance: 10.0 }
    }
}

impl CameraController for ThirdPersonCam {
    fn tick(&mut self, _dt: f32, camera: &mut Camera, target_transform: &Mat4) {
        // Align position
        let translation = target_transform.w_axis.truncate();
        let forward = -target_transform.z_axis.truncate().normalize();
        let target_camera_pos = translation - self.distance * forward;
        camera.position = target_camera_pos;

        // Align rotation: Using this over look_at to smoothen rotation and avoid numerical
        // instabilities
        let up = target_transform.y_axis.truncate().normalize();
        let right = forward.cross(up).normalize();
        let rotation_matrix = Mat3::from_cols(right, up, -forward);
        camera.set_rotation(Quat::from_mat3(&rotation_matrix));
    }
}
