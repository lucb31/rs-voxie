use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use winit::keyboard::KeyCode;

use crate::{
    cameras::camera::Camera, collision::query_sphere_cast, command_queue::CommandQueue,
    game::GameContext, logic::gun::Gun, meshes::sphere::SphereMesh, scenes::Renderer,
    voxels::VoxelWorld,
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
    mesh: SphereMesh,
    context: Rc<RefCell<GameContext>>,
    world: Rc<RefCell<VoxelWorld>>,

    gun: Gun,
}

const MAX_COLLIDE_BOUNCES: u32 = 3;
const SKIN_WIDTH: f32 = 0.015;

impl Player {
    pub fn new(
        gl: &Rc<glow::Context>,
        context: &Rc<RefCell<GameContext>>,
        world: &Rc<RefCell<VoxelWorld>>,
        command_queue: &Rc<RefCell<CommandQueue>>,
    ) -> Result<Player, Box<dyn Error>> {
        let mesh = SphereMesh::new(gl)?;
        Ok(Self {
            context: Rc::clone(context),
            gun: Gun::new(command_queue),
            mesh,
            pitch: 0.0,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            sensitivity: 0.02,
            speed: 15.0,
            velocity: Vec3::ZERO,
            yaw: 0.0,
            // TODO: Remove direct reference
            world: Rc::clone(world),
        })
    }

    pub fn tick(&mut self, dt: f32) {
        self.process_mouse_movement();
        self.process_keyboard();
        if self.velocity.length_squared() > 0.0001 {
            // If to avoid normalizing 0 vec
            let input_velocity = self.velocity.normalize() * dt * self.speed;
            let collision_adjusted_velocity =
                self.collide_and_slide(input_velocity, self.position, 0);
            let updated_position = self.position + collision_adjusted_velocity;
            // Ensure player cannot go out of bounds
            self.position = updated_position.clamp(Vec3::ONE, Vec3::ONE * 1000.0);
            self.velocity = Vec3::ZERO;
        }
        self.mesh.position = self.position;
        self.gun.tick(dt);
    }

    /// Collide and slide algorithm. Basic version. Based on
    /// https://www.youtube.com/watch?v=YR6Q7dUz2uk
    fn collide_and_slide(&self, vel: Vec3, pos: Vec3, depth: u32) -> Vec3 {
        if depth >= MAX_COLLIDE_BOUNCES {
            return Vec3::ZERO;
        }
        debug_assert!(vel.is_finite());
        let dist = vel.length() + SKIN_WIDTH;
        let vel_normalized = vel.normalize();
        let collision_test = query_sphere_cast(
            &self.world.borrow(),
            pos,
            self.mesh.radius - SKIN_WIDTH,
            vel_normalized,
            dist,
        );
        if let Some(collision) = collision_test {
            let mut snap_to_surface = vel_normalized * (collision.distance - SKIN_WIDTH);
            let leftover = vel - snap_to_surface;

            if snap_to_surface.length() <= SKIN_WIDTH {
                snap_to_surface = Vec3::ZERO;
            }

            let leftover_length = leftover.length();
            let projection_normalized = leftover.project_onto(collision.normal).normalize();
            debug_assert!(projection_normalized.is_finite());
            let projection = projection_normalized * leftover_length;
            return snap_to_surface
                + self.collide_and_slide(projection, pos + snap_to_surface, depth + 1);
        }
        vel
    }

    pub fn render(&mut self, cam: &Camera) {
        self.mesh.render(cam);
    }

    pub fn get_transform(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.position)
    }

    pub fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("Player")
            .size([300.0, 150.0], imgui::Condition::FirstUseEver)
            .position([600.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Position: {:.2}", self.position));
                ui.text(format!(
                    "Position (Chunk): {}",
                    self.world
                        .borrow()
                        .world_space_pos_to_chunk_space_pos(&self.position)
                ));
                ui.text(format!("Velocity: {:.2}", self.velocity));
                ui.slider("Player speed", 5.0, 50.0, &mut self.speed);
                ui.slider("Mouse sensitivity", 0.01, 0.03, &mut self.sensitivity);
            });
    }

    fn process_mouse_movement(&mut self) {
        let ctx = self.context.borrow();
        let mut input_state = ctx.input_state.borrow_mut();
        if !input_state.is_mouse_button_pressed(&winit::event::MouseButton::Left) {
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
        let transform = self.get_transform();
        let forward = (-transform.z_axis.xyz()).normalize();
        // let right = Vec3::Y.cross(pos_z_direction).normalize();

        let ctx = self.context.borrow();
        let input_state = ctx.input_state.borrow();
        let keys_pressed = &input_state.keys_pressed;
        for key in keys_pressed {
            match key {
                KeyCode::KeyW => self.velocity += forward,
                KeyCode::KeyS => self.velocity -= forward,
                KeyCode::Space => {
                    self.gun.fire(&transform);
                }
                _ => {}
            }
        }
    }
}
