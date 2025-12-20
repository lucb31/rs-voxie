use glam::{Mat3, Mat4, Quat, Vec3, Vec4Swizzles};

use crate::util::smooth_damp;

use super::camera::{Camera, CameraController};

pub struct ThirdPersonCam {
    distance: f32,
    position_smooth_time: f32,
    rotation_smooth_time: f32,
}

impl ThirdPersonCam {
    pub fn new() -> ThirdPersonCam {
        Self {
            distance: 10.0,
            position_smooth_time: 0.05,
            rotation_smooth_time: 0.08,
        }
    }
}

impl CameraController for ThirdPersonCam {
    fn tick(&mut self, dt: f32, camera: &mut Camera, target_transform: &Mat4) {
        // Smoothen position towards aligned with target forward + distance
        let target_position = target_transform.w_axis.xyz();
        let forward = (-target_transform.z_axis.xyz()).normalize();
        let target_camera_pos = target_position - self.distance * forward;
        let mut velocity = Vec3::ZERO;
        camera.position = smooth_damp(
            camera.position,
            target_camera_pos,
            &mut velocity,
            self.position_smooth_time,
            dt,
        );

        // Smoothen rotation towards aligned rotation with target
        let up = target_transform.y_axis.truncate().normalize();
        let right = forward.cross(up).normalize();
        let rotation_matrix = Mat3::from_cols(right, up, -forward);
        let target_quat = Quat::from_mat3(&rotation_matrix);
        camera.set_rotation(quat_exp_smooth(
            camera.get_rotation(),
            target_quat,
            self.rotation_smooth_time,
            dt,
        ));
    }
}

fn quat_exp_smooth(current: Quat, target: Quat, smooth_time: f32, dt: f32) -> Quat {
    let t = 1.0 - (-dt / smooth_time).exp();
    Quat::slerp(current, target, t)
}
