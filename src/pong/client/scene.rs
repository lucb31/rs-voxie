use crate::{
    cameras::component::CameraComponent,
    collision::system_collisions,
    input::InputState,
    log_err,
    network::{Authority, NetworkWorld, SnapshotManager},
    pong::{ClientProtocol, network::client::ClientMessage},
    systems::physics::{Transform, system_movement},
};
use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Mat4, Vec3};
use glow::HasContext;
use hecs::World;
use imgui::Ui;

use crate::scenes::Scene;

use super::{
    ball::PongBall,
    boundary::spawn_boundaries,
    paddle::system_paddle_movement,
    player::{sample_player_input, sync_player_input},
    sync::client_handle_network_cmd,
};

pub(super) enum GameState {
    Initial,
    WaitingForOthers { player_slot: usize },
    Running { player_slot: usize },
    GameOver { winner: bool },
}

pub struct PongScene {
    game_state: GameState,
    world: NetworkWorld,

    // Networking
    client_protocol: ClientProtocol,
    snapshot_manager: SnapshotManager,

    input_state: Rc<RefCell<InputState>>,
}

impl PongScene {
    pub fn new(
        client_protocol: ClientProtocol,
        input_state: Rc<RefCell<InputState>>,
    ) -> Result<PongScene, Box<dyn Error>> {
        let mut world = NetworkWorld::new();
        // Spawn camera directly into world -> No replication
        let scale_y = 3.5;
        let scale_x = scale_y * 16.0 / 9.0;
        let projection =
            Mat4::orthographic_rh_gl(-scale_x, scale_x, -scale_y, scale_y, -scale_y, scale_y);
        world.get_world_mut().spawn((
            Transform(Mat4::from_translation(Vec3::X * 3.5)),
            CameraComponent { projection },
        ));

        // Spawn boundaries directly into world -> No replication required
        let width = 5.0;
        let height = 5.0;
        spawn_boundaries(world.get_world_mut(), width, height);
        Ok(Self {
            snapshot_manager: SnapshotManager::new(),
            client_protocol,
            input_state,
            game_state: GameState::Initial,
            world,
        })
    }

    fn request_start_round(&mut self) {
        log_err!(
            self.client_protocol.send_cmd(ClientMessage::RequestJoin),
            "Unable to send start command to server: {err}"
        );
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

    fn overlay_ui(&mut self, ui: &mut Ui) {
        let io = ui.io();
        let window_size = [250.0, 100.0];
        let centered_pos = [
            (io.display_size[0] - window_size[0]) * 0.5,
            (io.display_size[1] - window_size[1]) * 0.5,
        ];
        let button_size = [120.0, 35.0];
        ui.window("Join")
            .size(window_size, imgui::Condition::FirstUseEver)
            .position(centered_pos, imgui::Condition::FirstUseEver)
            .build(|| match self.game_state {
                GameState::Initial => {
                    if ui.button_with_size("Join game", button_size) {
                        self.request_start_round();
                    }
                }
                GameState::WaitingForOthers { player_slot } => {
                    ui.text(format!(
                        "Connected as Player {player_slot}, waiting for others..."
                    ));
                }
                GameState::GameOver { winner } => {
                    ui.text(if winner {
                        "You've won!".to_string()
                    } else {
                        "You've lost".to_string()
                    });
                    if ui.button_with_size("Play again", button_size) {
                        self.request_start_round();
                    }
                }
                _ => panic!("Trying to display overlay for unknown game state"),
            });
    }
}

impl Scene for PongScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        self.client_protocol.render_ui(ui);
        if !matches!(self.game_state, GameState::Running { player_slot }) {
            self.overlay_ui(ui);
        } else {
            self.ball_ui(ui);
        }
    }

    fn get_title(&self) -> String {
        "Pong".to_string()
    }

    fn tick(&mut self, dt: f32) {
        while let Some(cmd) = self.client_protocol.try_recv() {
            client_handle_network_cmd(
                &mut self.world,
                cmd,
                &mut self.game_state,
                &mut self.snapshot_manager,
                &self.client_protocol,
            );
        }
        if matches!(self.game_state, GameState::Running { player_slot }) {
            sample_player_input(self.world.get_world_mut(), &self.input_state.borrow());
            sync_player_input(&self.world, &self.client_protocol);
            let collisions = system_collisions(self.world.get_world_mut());
            system_paddle_movement(self.world.get_world_mut(), &collisions);
            system_movement(self.world.get_world_mut(), dt);
        }

        self.client_protocol.tick();
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        if let Some(client_id) = self.client_protocol.get_client_id() {
            self.snapshot_manager.tick(&mut self.world, client_id);
        }
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn start(&mut self) {}

    fn get_world(&self) -> Option<&World> {
        Some(self.world.get_world())
    }
}
