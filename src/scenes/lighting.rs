use std::{cell::RefCell, error::Error, rc::Rc, time::Duration};

use glam::{Mat4, Vec3};
use glow::HasContext;
use hecs::World;
use log::error;

use crate::{
    cameras::{
        component::{CameraComponent, spawn_camera},
        orbit::BlenderOrbitCamera,
    },
    input::InputState,
    scenes::GuiScene,
    systems::physics::Transform,
    voxie::player::squid::spawn_squid,
};

use super::scene::BaseScene;

/// Used to debug & visualize lighting shaders & algorithms
pub struct LightingScene {
    cam: BlenderOrbitCamera,
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
        spawn_squid(&mut world, player_pos);

        // Setup camera
        spawn_camera(&mut world, Mat4::IDENTITY);
        let cam = BlenderOrbitCamera::new(Vec3::ZERO, 15.0);

        Ok(Self {
            cam,
            input_state,
            last_mouse_position: (0.0, 0.0),
            world,
        })
    }

    // Orbit camera around origin
    fn process_mouse_movement(&mut self) {
        let input_state = self.input_state.borrow();
        let current = input_state.get_mouse_position_f32();
        let delta = (
            self.last_mouse_position.0 - current.0,
            self.last_mouse_position.1 - current.1,
        );
        self.last_mouse_position = current;

        if let Some((_entity, (_cam, transform))) = self
            .world
            .query::<(&CameraComponent, &mut Transform)>()
            .iter()
            .next()
        {
            self.cam.orbit(delta.0, delta.1);
            transform.0 = self.cam.camera_transform();
        } else {
            error!("Could not apply mouse input: Cam not found")
        }
    }
}

impl BaseScene for LightingScene {
    fn get_title(&self) -> String {
        "Lighting Test".to_string()
    }

    fn tick(&mut self, _dt: f32) {
        self.process_mouse_movement();
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

    fn render(&mut self, _gl: &glow::Context, _dt: Duration) {}

    fn render_ui(&mut self, _ui: &mut imgui::Ui) {}
}
