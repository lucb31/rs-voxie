use crate::{
    input::InputState, logic::GameContext, renderer::ECSRenderer, systems::physics::system_movement,
};
use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use hecs::World;
use imgui::Ui;
use log::info;

use crate::{cameras::camera::Camera, scenes::Scene};

use super::player::{spawn_player, system_pong_movement};

pub struct PongScene {
    gl: Rc<glow::Context>,
    ecs: World,
    ecs_renderer: ECSRenderer,
    context: GameContext,

    camera: Camera,
}

impl PongScene {
    pub fn new(
        gl: &Rc<glow::Context>,
        input_state: &Rc<RefCell<InputState>>,
    ) -> Result<PongScene, Box<dyn Error>> {
        let player_position = Vec3::new(-0.0, 0.0, 0.0);
        // Camera setup
        let mut camera = Camera::new();
        let scale = 2.5;
        camera.set_projection(Mat4::orthographic_rh_gl(
            -scale, scale, -scale, scale, -scale, scale,
        ));

        // Setup context
        let context = GameContext::new(input_state.clone());

        // Prepare rendering
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let mut ecs = World::new();
        spawn_player(&mut ecs, player_position);

        Ok(Self {
            camera,
            context,
            ecs,
            gl: Rc::clone(gl),
            ecs_renderer: ECSRenderer::new(gl)?,
        })
    }
}

impl Scene for PongScene {
    fn render_ui(&mut self, ui: &mut Ui) {}

    fn get_title(&self) -> String {
        "Pong".to_string()
    }

    fn tick(&mut self, dt: f32) {
        self.context.tick();
        system_pong_movement(&mut self.ecs, &self.context.input_state.borrow(), dt);
        system_movement(&mut self.ecs, dt);
    }

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.ecs_renderer.render(&mut self.ecs, &self.camera);
    }

    fn start(&mut self) {
        info!("Starting pong scene...");
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }
}
