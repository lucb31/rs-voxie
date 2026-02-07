use std::{cell::RefCell, error::Error, rc::Rc, time::Duration};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use hecs::World;
use log::error;

use crate::{
    cameras::component::spawn_camera,
    input::InputState,
    scenes::GuiScene,
    systems::physics::{Transform, system_movement},
    voxie::player::{Player, spawn_player},
};

use super::scene::BaseScene;

/// Used to debug & visualize lighting shaders & algorithms
pub struct LightingScene {
    input_state: Rc<RefCell<InputState>>,
    last_mouse_position: (f32, f32),
    world: World,
}

impl LightingScene {
    pub fn new(
        _gl: &Rc<glow::Context>,
        input_state: Rc<RefCell<InputState>>,
    ) -> Result<LightingScene, Box<dyn Error>> {
        let mut world = World::new();

        // Spawn something to look at
        let player_pos = Vec3::ZERO;
        spawn_player(&mut world, player_pos);

        // Spawn camera
        let position = Vec3::new(0.0, 0.0, 10.0);
        spawn_camera(&mut world, Mat4::from_translation(position));

        Ok(Self {
            input_state,
            last_mouse_position: (0.0, 0.0),
            world,
        })
    }

    // Simple object rotation to mimic arcball
    fn process_mouse_movement(&mut self) {
        let input_state = self.input_state.borrow();
        let current = input_state.get_mouse_position_f32();
        let delta = (
            self.last_mouse_position.0 - current.0,
            self.last_mouse_position.1 - current.1,
        );
        self.last_mouse_position = current;
        let dx = delta.0;
        let dy = delta.1;
        let sensitivity = 0.005;
        let yaw = Quat::from_rotation_y(dx * sensitivity);
        let pitch = Quat::from_rotation_x(dy * sensitivity);

        if let Some((_entity, (_player, transform))) = self
            .world
            .query::<(&Player, &mut Transform)>()
            .iter()
            .next()
        {
            let (scale, rot, trans) = Mat4::to_scale_rotation_translation(&transform.0);
            transform.0 = Mat4::from_scale_rotation_translation(scale, rot * yaw * pitch, trans);
        } else {
            error!("Could not apply mouse input: Player not found")
        }
    }
}

impl BaseScene for LightingScene {
    fn get_title(&self) -> String {
        "Lighting Test".to_string()
    }

    fn tick(&mut self, dt: f32) {
        self.process_mouse_movement();
        system_movement(&mut self.world, dt);
    }

    fn start(&mut self) {}
    fn get_world(&self) -> Option<&hecs::World> {
        Some(&self.world)
    }
}
impl GuiScene for LightingScene {
    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn render(&mut self, gl: &glow::Context, _dt: Duration) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn render_ui(&mut self, _ui: &mut imgui::Ui) {}
}
