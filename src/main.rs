use std::{env, sync::mpsc};

use application::Application;
use log::{error, info};
use network::{HeadlessSimulation, NetworkClient, NetworkServer, ServerUpstreamPayload};
use pong::{BincodeCodec, ClientProtocol, PongServerScene, ServerProtocol};

mod application;
mod cameras;
mod collision;
mod command_queue;
mod cube;
mod input;
mod logic;
mod meshes;
mod network;
mod octree;
mod pong;
mod renderer;
mod scenes;
mod systems;
mod util;
mod voxels;

#[derive(Debug)]
enum SceneSelection {
    Benchmark,
    Collision,
    Game,
    Lighting,
    PongClient,
    PongServer,
}

impl SceneSelection {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "benchmark" => Some(SceneSelection::Benchmark),
            "collision" => Some(SceneSelection::Collision),
            "game" => Some(SceneSelection::Game),
            "lighting" => Some(SceneSelection::Lighting),
            "pong" => Some(SceneSelection::PongClient),
            "pong-server" => Some(SceneSelection::PongServer),
            _ => None,
        }
    }
}

struct CliArgs {
    scene: Option<SceneSelection>,
    headless: bool,
    server_address: String,
}

impl CliArgs {
    pub fn default() -> Self {
        Self {
            scene: Some(SceneSelection::Game),
            headless: false,
            server_address: "127.0.0.1:7777".to_string(),
        }
    }
}

fn parse_args() -> CliArgs {
    let args: Vec<String> = env::args().collect();

    let mut result = CliArgs::default();
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--scene" {
            if i + 1 < args.len() {
                if let Some(parsed_scene) = SceneSelection::from_str(&args[i + 1]) {
                    result.scene = Some(parsed_scene);
                } else {
                    error!(
                        "Invalid scene: '{}'. Valid options are: benchmark, game, collision, lighting",
                        args[i + 1]
                    );
                    std::process::exit(1);
                }
                i += 1; // skip next
            } else {
                error!("Expected value after --scene");
                std::process::exit(1);
            }
        } else if args[i] == "--headless" {
            result.headless = true;
        } else if args[i] == "--server-address" {
            if i + 1 < args.len() {
                result.server_address = args[i + 1].to_string();
                i += 1; // skip next
            } else {
                error!("Expected value after --server-address");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    result
}

fn main() {
    env_logger::init();
    let cli_args = parse_args();

    // Server mode
    if cli_args.headless {
        // Setup transport layer
        let mut server = NetworkServer::new();
        let (upstream_tx, upstream_rx) = mpsc::channel::<ServerUpstreamPayload>();
        server
            .serve("0.0.0.0:7777", upstream_tx)
            .expect("Could not serve");

        // Setup protocol layer
        let protocol = ServerProtocol::<BincodeCodec>::new(server, upstream_rx)
            .expect("Could not init protocol");

        let scene = PongServerScene::new(protocol).expect("Could not initialize pong scene");
        let mut simulation = HeadlessSimulation::new(Box::new(scene));
        simulation.run();
    } else {
        // Client mode
        let scene = cli_args.scene.expect("No scene selected");
        // Setup application
        let mut app = Application::new("Voxie").expect("Could not setup application");
        let gl_ctx = app.gl_context().clone();

        // Setup scene(s) to render
        match scene {
            SceneSelection::Benchmark => {
                info!("Running benchmark scene...");
                app.max_scene_duration_secs = 2.0;
                for size_power in 2..6 {
                    let base: usize = 2;
                    let world_size = base.pow(size_power);
                    let mut scene = scenes::BenchmarkScene::new(&gl_ctx, world_size)
                        .expect("Unable to initialize scene");
                    scene.title = format!("{world_size}x{world_size}x{world_size} cubes");
                    app.add_scene(Box::new(scene));
                }
            }
            SceneSelection::Game => {
                info!("Running game scene...");
                let scene = scenes::GameScene::new(&gl_ctx, app.input_state.clone())
                    .expect("Unable to initialize scene");
                app.add_scene(Box::new(scene));
            }
            SceneSelection::Collision => {
                let scene = scenes::collision::CollisionScene::new(&gl_ctx)
                    .expect("Could not init collision scene");
                app.add_scene(Box::new(scene));
            }
            SceneSelection::Lighting => {
                let scene = scenes::LightingScene::new(&gl_ctx, app.input_state.clone())
                    .expect("Could not init lighting scene");
                app.add_scene(Box::new(scene));
            }
            SceneSelection::PongClient => {
                // NETWORKING
                // Setup transport layer
                let (downstream_bytes_tx, downstream_bytes_rx) = mpsc::channel::<Vec<u8>>();
                let client = NetworkClient::new(&cli_args.server_address, downstream_bytes_tx)
                    .expect("Could not initialize transport layer");
                // Setup protocol layer
                let protocol = ClientProtocol::new(downstream_bytes_rx, client)
                    .expect("Could not init client proto");

                // Setup scene
                let scene = pong::PongScene::new(protocol, app.input_state.clone())
                    .expect("Could not init pong scene");
                app.add_scene(Box::new(scene));
            }
            SceneSelection::PongServer => {
                // Setup transport layer
                let mut server = NetworkServer::new();
                let (upstream_tx, upstream_rx) = mpsc::channel::<ServerUpstreamPayload>();
                server
                    .serve("0.0.0.0:7777", upstream_tx)
                    .expect("Could not serve");

                // Setup protocol layer
                let protocol = ServerProtocol::<BincodeCodec>::new(server, upstream_rx)
                    .expect("Could not init protocol");

                let scene =
                    PongServerScene::new(protocol).expect("Could not initialize pong server scene");
                app.add_scene(Box::new(scene));
            }
        }

        app.run().expect("Failed to run application");
    }
}
