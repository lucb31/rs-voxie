use crate::{
    collision::{CollisionEvent, system_collisions},
    input::InputState,
    logic::GameContext,
    renderer::ECSRenderer,
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
    ball::{PongBall, bounce_balls, despawn_balls, spawn_ball},
    boundary::{despawn_boundaries, spawn_boundaries},
    paddle::{despawn_paddles, system_paddle_movement},
    player::{spawn_player, system_player_input},
};

pub struct PongScene {
    gl: Rc<glow::Context>,
    world: World,
    ecs_renderer: ECSRenderer,
    context: GameContext,

    camera: Camera,

    collisions: Vec<CollisionEvent>,
    game_over: bool,
}

impl PongScene {
    pub fn new(
        gl: &Rc<glow::Context>,
        input_state: &Rc<RefCell<InputState>>,
    ) -> Result<PongScene, Box<dyn Error>> {
        // Setup camera
        let mut camera = Camera::new();
        let scale_y = 2.5;
        let scale_x = scale_y * 16.0 / 9.0;
        camera.set_projection(Mat4::orthographic_rh_gl(
            -scale_x, scale_x, -scale_y, scale_y, -scale_y, scale_y,
        ));

        // Setup context
        let context = GameContext::new(input_state.clone());
        let world = World::new();

        Ok(Self {
            camera,
            context,
            world,
            gl: Rc::clone(gl),
            ecs_renderer: ECSRenderer::new(gl)?,
            collisions: Vec::new(),
            game_over: true,
        })
    }

    fn end_round(&mut self) {
        info!("Ending round");
        despawn_balls(&mut self.world);
        despawn_paddles(&mut self.world);
        despawn_boundaries(&mut self.world);
        self.game_over = true;
    }

    fn start_round(&mut self) {
        info!("Starting round");
        spawn_player(&mut self.world, Vec3::new(-2.3, 0.0, 0.0));
        spawn_ai(&mut self.world, Vec3::new(2.3, 0.0, 0.0));
        let width = 5.0;
        let height = 5.0;
        spawn_boundaries(&mut self.world, width, height);
        spawn_ball(&mut self.world);
        self.game_over = false;
    }

    fn ball_ui(&mut self, ui: &mut Ui) {
        let mut ball_query = self.world.query::<&PongBall>();
        let (_ball_entity, ball) = match ball_query.iter().next() {
            Some(b) => b,
            None => return,
        };
        ui.window("Ball")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([300.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Bounces: {}", ball.bounces));
                ui.text(format!("Speed: {}", ball.speed));
            });
    }

    fn collision_ui(&mut self, ui: &mut Ui) {
        ui.window("Collision events")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([450.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("#: {}", self.collisions.len()));
            });
    }

    fn start_game_ui(&mut self, ui: &mut Ui) {
        let io = ui.io();
        let window_size = [150.0, 100.0];
        let centered_pos = [
            (io.display_size[0] - window_size[0]) * 0.5,
            (io.display_size[1] - window_size[1]) * 0.5,
        ];
        let button_size = [120.0, 35.0];
        ui.window("Start game")
            .size(window_size, imgui::Condition::FirstUseEver)
            .position(centered_pos, imgui::Condition::FirstUseEver)
            .build(|| {
                if ui.button_with_size("Start new game (SPACE)", button_size) {
                    self.start_round();
                }
            });
    }
}

impl Scene for PongScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        if self.game_over {
            self.start_game_ui(ui);
        } else {
            self.ball_ui(ui);
            self.collision_ui(ui);
        }
    }

    fn get_title(&self) -> String {
        "Pong".to_string()
    }

    fn tick(&mut self, dt: f32) {
        self.context.tick();
        if self.game_over {
            // Press SPACE to start new round
            if self
                .context
                .input_state
                .borrow()
                .is_key_pressed(&winit::keyboard::KeyCode::Space)
            {
                self.start_round();
            }
            return;
        }
        system_player_input(&mut self.world, &self.context.input_state.borrow());
        system_ai(&mut self.world, dt);

        // Collision systems
        self.collisions = system_collisions(&mut self.world);
        let game_over = bounce_balls(&mut self.world, &self.collisions);
        if game_over {
            self.end_round();
        }
        system_paddle_movement(&mut self.world, &self.collisions);

        // Physics simulation
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
