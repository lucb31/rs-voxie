use glam::{Mat4, Quat, Vec3};

use super::camera::Camera;

pub struct FirstPersonCam {
    offset: Vec3,
    smooth_speed: f32,
}

trait CameraController {}

impl FirstPersonCam {
    pub fn new() -> FirstPersonCam {
        let camera_height = 5.0;
        let camera_distance = 5.0;
        Self {
            offset: Vec3::new(0.0, camera_height, -camera_distance),
            smooth_speed: 0.125,
        }
    }

    pub fn tick(&mut self, dt: f32, camera: &mut Camera, target_transform: &Mat4) {
        let translation = target_transform.w_axis.truncate();
        let rotation = Quat::from_mat4(target_transform);
        camera.position = translation;
        camera.set_rotation(rotation);
        return;
        // Left-off: Actual follow cam with offset & look at
        let target_camera_pos = translation + self.offset;
        camera.position = target_camera_pos;
        camera.look_at(translation);
    }
}
