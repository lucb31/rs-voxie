use std::env;

use application::Application;
use scene::Scene;

mod application;
mod benchmark;
mod cameras;
mod collision;
mod cube;
mod game;
mod meshes;
mod octree;
mod player;
mod renderer;
mod scene;
mod scenes;
mod util;
mod voxel;
mod voxels;
mod world;

#[derive(Debug)]
enum SceneSelection {
    Benchmark,
    Game,
    Collision,
}

impl SceneSelection {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "benchmark" => Some(SceneSelection::Benchmark),
            "game" => Some(SceneSelection::Game),
            "collision" => Some(SceneSelection::Collision),
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
                    eprintln!(
                        "Invalid scene: '{}'. Valid options are: benchmark, game, collision",
                        args[i + 1]
                    );
                    std::process::exit(1);
                }
                i += 1; // skip next
            } else {
                eprintln!("Expected value after --scene");
                std::process::exit(1);
            }
        }
        i += 1;
    }

    scene
}

fn main() {
    let selected_scene = parse_args();
    // Setup application
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let gl_ctx = app.gl_context().clone();

    // Setup scene(s) to render
    let mut scenes: Vec<Box<dyn Scene>> = vec![];
    match selected_scene {
        SceneSelection::Benchmark => {
            println!("Running benchmark scene...");
            app.max_scene_duration_secs = 2.0;
            for size_power in 2..6 {
                let base: usize = 2;
                let world_size = base.pow(size_power);
                let mut scene = scene::BenchmarkScene::new(gl_ctx.clone(), world_size)
                    .expect("Unable to initialize scene");
                scene.title = format!("{world_size}x{world_size}x{world_size} cubes");
                scenes.push(Box::new(scene));
            }
            // Start with the easy scenes
            scenes.reverse();
        }
        SceneSelection::Game => {
            println!("Running game scene...");
            let scene = game::GameScene::new(gl_ctx.clone(), app.input_state.clone())
                .expect("Unable to initialize scene");
            scenes.push(Box::new(scene));
        }
        SceneSelection::Collision => {
            let scene = scenes::collision::CollisionScene::new(gl_ctx.clone())
                .expect("Could not init collision scene");
            scenes.push(Box::new(scene));
        }
    }

    app.run(scenes).expect("Failed to run application");
}
