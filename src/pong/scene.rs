use crate::{
    collision::system_collisions, input::InputState, logic::GameContext, renderer::ECSRenderer,
    systems::physics::system_movement,
};
use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Mat4, Vec3};
use glow::HasContext;
use hecs::World;
use imgui::Ui;
use log::info;

use crate::{cameras::camera::Camera, scenes::Scene};

use super::{
    ai::{spawn_ai, system_ai},
    ball::{bounce_ball, spawn_ball},
    boundary::spawn_boundaries,
    paddle::system_paddle_movement,
    player::{spawn_player, system_player_input},
};

pub struct PongScene {
    gl: Rc<glow::Context>,
    world: World,
    ecs_renderer: ECSRenderer,
    context: GameContext,

    camera: Camera,
}

impl PongScene {
    pub fn new(
        gl: &Rc<glow::Context>,
        input_state: &Rc<RefCell<InputState>>,
    ) -> Result<PongScene, Box<dyn Error>> {
        let player_position = Vec3::new(-2.0, 0.0, 0.0);
        // Camera setup
        let mut camera = Camera::new();
        let scale_y = 2.5;
        let scale_x = scale_y * 16.0 / 9.0;
        camera.set_projection(Mat4::orthographic_rh_gl(
            -scale_x, scale_x, -scale_y, scale_y, -scale_y, scale_y,
        ));

        // Setup context
        let context = GameContext::new(input_state.clone());

        let mut world = World::new();
        spawn_player(&mut world, player_position);
        spawn_ai(&mut world, Vec3::new(2.0, 0.0, 0.0));
        let width = 5.0;
        let height = 5.0;
        spawn_boundaries(&mut world, width, height);
        spawn_ball(&mut world);

        Ok(Self {
            camera,
            context,
            world,
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
        system_player_input(&mut self.world, &self.context.input_state.borrow());
        system_ai(&mut self.world);

        let collisions = system_collisions(&mut self.world);
        system_paddle_movement(&mut self.world, &collisions);
        bounce_ball(&mut self.world, &collisions);

        system_movement(&mut self.world, dt);
    }

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.ecs_renderer.render(&mut self.world, &self.camera);
    }

    fn start(&mut self) {
        info!("Starting pong scene...");
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }
}
