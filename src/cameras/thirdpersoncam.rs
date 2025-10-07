use glam::{Mat4, Quat, Vec3};

use super::camera::{Camera, CameraController};

pub struct ThirdPersonCam {
    distance: f32,
}

impl ThirdPersonCam {
    pub fn new() -> ThirdPersonCam {
        Self { distance: 10.0 }
    }
}

impl CameraController for ThirdPersonCam {
    fn tick(&mut self, _dt: f32, camera: &mut Camera, target_transform: &Mat4) {
        let translation = target_transform.w_axis.truncate();
        let rotation = Quat::from_mat4(target_transform);
        let pos_z_direction = rotation * -Vec3::Z;
        let target_camera_pos = translation - self.distance * pos_z_direction;
        camera.position = target_camera_pos;
        camera.look_at(translation);
    }
}
