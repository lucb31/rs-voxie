use crate::{
    collision::{CollisionEvent, system_collisions},
    input::InputState,
    log_err,
    network::{EcsSynchronizer, JsonCodec, NetworkClient, NetworkCommand, NetworkScene},
    renderer::ECSRenderer,
    systems::physics::system_movement,
};
use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    sync::mpsc::{self},
};

use glam::Mat4;
use glow::HasContext;
use hecs::World;
use imgui::Ui;
use log::info;

use crate::{cameras::camera::Camera, scenes::Scene};

use super::{
    ai::system_ai,
    ball::{PongBall, bounce_balls, despawn_balls, spawn_ball},
    boundary::{despawn_boundaries, spawn_boundaries},
    paddle::{despawn_paddles, system_paddle_movement},
    player::{spawn_player, system_player_input},
};

pub struct PongScene {
    camera: Camera,
    collisions: Vec<CollisionEvent>,
    game_over: bool,
    world: World,

    // Client only
    // TODO: Left off here: Probably even more systems are client only
    // Might be a good idea to wrap in a "ClientState" struct
    input_state: Option<Rc<RefCell<InputState>>>,
    ecs_renderer: Option<ECSRenderer>,
    gl: Option<Rc<glow::Context>>,

    // Client-networking
    client: Option<NetworkClient<JsonCodec>>,
    ecs_sync: Option<EcsSynchronizer>,
}

impl PongScene {
    pub fn new() -> Result<PongScene, Box<dyn Error>> {
        // Setup camera
        let mut camera = Camera::new();
        let scale_y = 2.5;
        let scale_x = scale_y * 16.0 / 9.0;
        camera.set_projection(Mat4::orthographic_rh_gl(
            -scale_x, scale_x, -scale_y, scale_y, -scale_y, scale_y,
        ));

        // Setup ecs
        let world = World::new();

        Ok(Self {
            camera,
            client: None,
            ecs_sync: None,
            collisions: Vec::new(),
            input_state: None,
            gl: None,
            ecs_renderer: None,
            game_over: true,
            world,
        })
    }

    pub fn setup_rendering(
        &mut self,
        gl: &Rc<glow::Context>,
        input_state: &Rc<RefCell<InputState>>,
    ) {
        self.ecs_renderer = ECSRenderer::new(gl).ok();
        self.gl = Some(Rc::clone(gl));
        self.input_state = Some(Rc::clone(input_state));
    }

    pub fn setup_networking(&mut self) {
        let (tx, rx) = mpsc::channel::<NetworkCommand>();
        self.ecs_sync = Some(EcsSynchronizer::new(rx));
        self.client = Some(
            NetworkClient::<JsonCodec>::new(&"127.0.0.1:8080", tx)
                .expect("Unable to connect to server"),
        );
    }

    fn end_round(&mut self) {
        info!("Ending round");
        despawn_balls(&mut self.world);
        despawn_paddles(&mut self.world);
        despawn_boundaries(&mut self.world);
        self.game_over = true;
    }

    fn request_start_round(&mut self) {
        if let Some(client) = &mut self.client {
            log_err!(
                client.send_cmd(NetworkCommand::ClientStartRound),
                "Unable to send start command to server: {err}"
            );
        }
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
                    self.request_start_round();
                }
            });
    }
}

impl Scene for PongScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        if let Some(client) = &mut self.client {
            client.render_ui(ui);
        }
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
        if let Some(ecs_sync) = &mut self.ecs_sync {
            ecs_sync.sync(&mut self.world);
        }
        if self.game_over {
            return;
        }
        if let Some(input_state) = &self.input_state {
            system_player_input(&mut self.world, &input_state.borrow());
        }
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
        if let Some(gl) = &self.gl {
            unsafe {
                gl.clear_color(0.05, 0.05, 0.1, 1.0);
                gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            }
            self.ecs_renderer
                .as_mut()
                .unwrap()
                .render(&mut self.world, &self.camera);
        }
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn start(&mut self) {}
}

impl NetworkScene for PongScene {
    fn get_world(&mut self) -> &mut World {
        &mut self.world
    }

    // TODO: Next steps
    // Distinguish ServerNetworkCommands from ClientNetworkCommands
    // More abstract spawn logic
    // fix Game over state
    // - More server game logic (spawn paddles, collision, etc)
    //
    // Approach to scene loading / start:
    // Init all entities on start on client & server
    // Init game_over to true
    // On client button click: Send command; Server updates will come in
    // Might be useful to have a paused state: THere's no point syncing game
    // & input state when scene is not started yet
    fn start_match(&mut self) {
        // Receive entities
        // Attach rendering to entities
        info!("Starting pong match...");
        let width = 5.0;
        let height = 5.0;
        spawn_boundaries(&mut self.world, width, height);
        spawn_ball(&mut self.world);
        self.game_over = false;

        //        spawn_player(&mut self.world, Vec3::new(-2.3, 0.0, 0.0));
        //        spawn_ai(&mut self.world, Vec3::new(2.3, 0.0, 0.0));
    }

    fn game_over(&self) -> bool {
        self.game_over
    }
}
