use std::env;

use application::Application;
use log::{error, info};

mod application;
mod benchmark;
mod cameras;
mod collision;
mod cube;
mod game;
mod input;
mod meshes;
mod metrics;
mod octree;
mod player;
mod renderer;
mod scene;
mod scenes;
mod util;
mod voxels;

#[derive(Debug)]
enum SceneSelection {
    Benchmark,
    Collision,
    Game,
    Lighting,
}

impl SceneSelection {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "benchmark" => Some(SceneSelection::Benchmark),
            "collision" => Some(SceneSelection::Collision),
            "game" => Some(SceneSelection::Game),
            "lighting" => Some(SceneSelection::Lighting),
            _ => None,
        }
    }
}

fn parse_args() -> SceneSelection {
    let args: Vec<String> = env::args().collect();
    let mut scene = SceneSelection::Game; // default

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--scene" {
            if i + 1 < args.len() {
                if let Some(parsed_scene) = SceneSelection::from_str(&args[i + 1]) {
                    scene = parsed_scene;
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
        }
        i += 1;
    }

    scene
}

fn main() {
    env_logger::init();
    let selected_scene = parse_args();
    // Setup application
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let gl_ctx = app.gl_context().clone();

    // Setup scene(s) to render
    match selected_scene {
        SceneSelection::Benchmark => {
            info!("Running benchmark scene...");
            app.max_scene_duration_secs = 2.0;
            for size_power in 2..6 {
                let base: usize = 2;
                let world_size = base.pow(size_power);
                let mut scene = scene::BenchmarkScene::new(gl_ctx.clone(), world_size)
                    .expect("Unable to initialize scene");
                scene.title = format!("{world_size}x{world_size}x{world_size} cubes");
                app.add_scene(Box::new(scene));
            }
        }
        SceneSelection::Game => {
            info!("Running game scene...");
            let scene = game::GameScene::new(gl_ctx.clone(), app.input_state.clone())
                .expect("Unable to initialize scene");
            app.add_scene(Box::new(scene));
        }
        SceneSelection::Collision => {
            let scene = scenes::collision::CollisionScene::new(gl_ctx.clone())
                .expect("Could not init collision scene");
            app.add_scene(Box::new(scene));
        }
        SceneSelection::Lighting => {
            let scene = scenes::LightingScene::new(gl_ctx.clone(), app.input_state.clone())
                .expect("Could not init lighting scene");
            app.add_scene(Box::new(scene));
        }
    }

    app.run().expect("Failed to run application");
}
