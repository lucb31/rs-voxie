use glam::{Mat4, Quat};

use super::camera::{Camera, CameraController};

pub struct FirstPersonCam {}

impl FirstPersonCam {
    pub fn new() -> FirstPersonCam {
        Self {}
    }
}

impl CameraController for FirstPersonCam {
    fn tick(&mut self, _dt: f32, camera: &mut Camera, target_transform: &Mat4) {
        let translation = target_transform.w_axis.truncate();
        let rotation = Quat::from_mat4(target_transform);
        camera.position = translation;
        camera.set_rotation(rotation);
    }
}
