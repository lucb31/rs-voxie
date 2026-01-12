use crate::{
    collision::system_collisions,
    input::InputState,
    log_err,
    network::{NetworkWorld, SnapshotManager},
    pong::{
        ClientProtocol,
        common::{
            ball::PongBall,
            paddle::{PaddleControl, system_paddle_movement},
            setup_static_entities,
        },
        network::{client::ClientMessage, input::ClientInputBuffer},
    },
    scenes::scene::BaseScene,
    systems::physics::system_movement,
};
use std::{
    cell::RefCell,
    error::Error,
    rc::Rc,
    time::{Duration, Instant},
};

use glow::HasContext;
use hecs::World;
use imgui::Ui;
use log::{debug, info};

use crate::scenes::GuiScene;

use super::{
    player::{apply_player_input, assemble_input_sync_cmd, sample_input},
    sync::client_handle_network_cmd,
};

pub(super) struct GameOverTransition {
    server_tick: u32,
    loosing_player_slot: usize,
}

impl GameOverTransition {
    pub(super) fn new(server_tick: u32, loosing_player_slot: usize) -> GameOverTransition {
        Self {
            server_tick,
            loosing_player_slot,
        }
    }
}

pub(super) enum GameState {
    // Initial state after loading scene and joining first game
    Initial,
    // Player joined & waits for others to fill the lobby
    WaitingForOthers {
        player_slot: usize,
    },
    // Game in progress
    Running {
        player_slot: usize,
        started_at_server_tick: u32,
        // Once game-over signal received from server will continue simulation until transition to
        // game over
        end_signal: Option<GameOverTransition>,
    },
    // Game over, UI to rejoin
    GameOver {
        winner: bool,
    },
}

pub struct PongScene {
    game_state: GameState,
    world: NetworkWorld,

    // Networking
    client_protocol: ClientProtocol,
    snapshot_manager: SnapshotManager,

    input_state: Rc<RefCell<InputState>>,
    input_buffer: ClientInputBuffer,
}

impl PongScene {
    pub fn new(
        client_protocol: ClientProtocol,
        input_state: Rc<RefCell<InputState>>,
    ) -> Result<PongScene, Box<dyn Error>> {
        let mut world = NetworkWorld::new();
        setup_static_entities(&mut world);
        Ok(Self {
            snapshot_manager: SnapshotManager::new(),
            client_protocol,
            input_state,
            game_state: GameState::Initial,
            world,
            input_buffer: ClientInputBuffer::new(),
        })
    }

    fn request_start_round(&mut self) {
        log_err!(
            self.client_protocol.send_cmd(ClientMessage::RequestJoin),
            "Unable to send start command to server: {err}"
        );
    }

    fn check_for_game_over(&mut self) {
        let GameState::Running {
            player_slot,
            end_signal: Some(transition),
            ..
        } = &self.game_state
        else {
            // Game currently not running
            return;
        };
        let approx = self.client_protocol.approx_server_tick(Instant::now());
        if approx < transition.server_tick {
            debug!(
                "Approximated server tick {}: Continue until {} ",
                approx, transition.server_tick
            );
            // Game should continue until game over tick is reached
            return;
        }
        debug!("Ending at {approx}: {}", transition.server_tick);

        info!("Game over tick reached. Ending client simulation");
        self.game_state = GameState::GameOver {
            winner: transition.loosing_player_slot != *player_slot,
        };
        log_err!(
            self.world.despawn_all::<&PongBall>(),
            "Could not despawn balls {err}"
        );
        log_err!(
            self.world.despawn_all::<&PaddleControl>(),
            "Could not despawn paddles {err}"
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
        let button_size = [160.0, 35.0];
        ui.window("Join")
            .size(window_size, imgui::Condition::FirstUseEver)
            .position(centered_pos, imgui::Condition::FirstUseEver)
            .build(|| match self.game_state {
                GameState::Initial => {
                    if self.client_protocol.is_connected() {
                        let btn = ui.button_with_size("Join game [SPACE]", button_size);
                        let keybind = ui.is_key_pressed(imgui::Key::Space);
                        if btn || keybind {
                            self.request_start_round();
                        }
                    } else {
                        ui.text("Server unavailable");
                    }
                }
                GameState::WaitingForOthers { player_slot } => {
                    ui.text_wrapped(format!(
                        "Connected as Player {player_slot}, waiting for others..."
                    ));
                }
                GameState::GameOver { winner } => {
                    ui.text(if winner {
                        "You've won!".to_string()
                    } else {
                        "You've lost".to_string()
                    });
                    let btn = ui.button_with_size("Play again [SPACE]", button_size);
                    let keybind = ui.is_key_pressed(imgui::Key::Space);
                    if btn || keybind {
                        self.request_start_round();
                    }
                }
                _ => panic!("Trying to display overlay for unknown game state"),
            });
    }
}

impl BaseScene for PongScene {
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
                &mut self.input_buffer,
            );
        }
        if let GameState::Running { .. } = &mut self.game_state {
            // Sample input
            sample_input(
                &mut self.input_buffer,
                &self.input_state.borrow(),
                self.client_protocol.get_client_tick(),
            );
            // Send to server
            let input_cmd = assemble_input_sync_cmd(&self.input_buffer);
            self.client_protocol
                .send_cmd(input_cmd)
                .expect("Could not send client input");
            // Apply input locally
            apply_player_input(self.world.get_world_mut(), &self.input_buffer);

            let collisions = system_collisions(self.world.get_world_mut());
            system_paddle_movement(self.world.get_world_mut(), &collisions);
            system_movement(self.world.get_world_mut(), dt);
            self.check_for_game_over();
        }

        self.client_protocol.tick();
    }

    fn start(&mut self) {}

    fn get_world(&self) -> Option<&World> {
        Some(self.world.get_world())
    }
}

impl GuiScene for PongScene {
    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn render(&mut self, gl: &glow::Context, dt: Duration) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        if let Some(client_id) = self.client_protocol.get_client_id() {
            self.snapshot_manager.tick(
                &mut self.world,
                client_id,
                &self.client_protocol.time_sync,
                dt,
            );
        }
    }

    fn render_ui(&mut self, ui: &mut Ui) {
        self.client_protocol.render_ui(ui);
        if !matches!(self.game_state, GameState::Running { .. }) {
            self.overlay_ui(ui);
        } else {
            self.ball_ui(ui);
        }
    }
}
