use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use winit::keyboard::KeyCode;

use crate::{
    cameras::camera::Camera, game::GameContext, meshes::sphere::SphereMesh, scene::Renderer,
};

fn quat_from_yaw_pitch(yaw: f32, pitch: f32) -> Quat {
    // Reconstruct the rotation from yaw and pitch
    let yaw_rotation = Quat::from_rotation_y(yaw);
    let pitch_rotation = Quat::from_rotation_x(pitch);

    // Combine yaw then pitch (Y * X)
    yaw_rotation * pitch_rotation
}

pub struct Player {
    pub position: Vec3,
    pub velocity: Vec3,
    rotation: Quat,
    pitch: f32,
    yaw: f32,

    // Movement speed
    pub speed: f32,
    // Sensitivity of yaw & pitch movement
    pub sensitivity: f32,
    gl: Rc<glow::Context>,
    mesh: SphereMesh,
    camera: Rc<RefCell<Camera>>,
    context: Rc<RefCell<GameContext>>,
}

impl Player {
    pub fn new(
        gl: Rc<glow::Context>,
        camera: Rc<RefCell<Camera>>,
        context: Rc<RefCell<GameContext>>,
    ) -> Result<Player, Box<dyn Error>> {
        let mesh = SphereMesh::new(gl.clone())?;
        Ok(Self {
            camera,
            context,
            gl,
            mesh,
            pitch: 0.0,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            sensitivity: 0.01,
            speed: 50.0,
            velocity: Vec3::ZERO,
            yaw: 0.0,
        })
    }

    pub fn tick(&mut self, dt: f32) {
        self.process_mouse_movement();
        self.process_keyboard();
        // Avoid normalizing 0 vec
        if self.velocity.length_squared() < 0.0001 {
            return;
        }
        let requested_movement = self.velocity.normalize() * dt * self.speed;
        self.position += requested_movement;
        self.velocity = Vec3::ZERO;
        self.mesh.position = self.position;
    }

    pub fn render(&mut self) {
        self.mesh.render(&self.gl, &self.camera.borrow());
    }

    pub fn get_transform(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position)
    }

    fn process_mouse_movement(&mut self) {
        let ctx = self.context.borrow();
        let mut input_state = ctx.input_state.borrow_mut();
        if !input_state.is_mouse_button_pressed(&winit::event::MouseButton::Middle) {
            return;
        }

        let delta = input_state.get_and_reset_mouse_moved();
        let dx = delta.0 as f32;
        let dy = delta.1 as f32;

        // Update yaw and pitch
        self.yaw -= dx * self.sensitivity;
        self.pitch -= dy * self.sensitivity;

        // Clamp pitch to [-89°, 89°] to prevent flipping
        let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01; // ~89.4°
        self.pitch = self.pitch.clamp(-pitch_limit, pitch_limit);

        self.rotation = quat_from_yaw_pitch(self.yaw, self.pitch);
    }

    fn process_keyboard(&mut self) {
        let pos_z_direction = self.rotation * -Vec3::Z;
        let right = Vec3::Y.cross(pos_z_direction).normalize();

        let ctx = self.context.borrow();
        let input_state = ctx.input_state.borrow();
        let keys_pressed = &input_state.keys_pressed;
        for key in keys_pressed {
            match key {
                KeyCode::KeyW => self.velocity += pos_z_direction,
                KeyCode::KeyS => self.velocity -= pos_z_direction,
                KeyCode::KeyA => self.velocity += right,
                KeyCode::KeyD => self.velocity -= right,
                _ => {}
            }
        }
    }
}
