use std::env;

use application::Application;
use scene::Scene;

mod application;
mod benchmark;
mod camera;
mod cube;
mod game;
mod meshes;
mod octree;
mod scene;
mod util;
mod voxel;
mod world;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let benchmark_enabled =
        args.contains(&"--benchmark".to_string()) || args.contains(&"-b".to_string());

    // Setup application
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let gl_ctx = app.gl_context().clone();

    // Setup scene(s) to render
    let mut scenes: Vec<Box<dyn Scene>> = vec![];
    if benchmark_enabled {
        println!("Running in benchmark mode...");
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
    } else {
        println!("Running game...");
        let scene = game::GameScene::new(gl_ctx.clone()).expect("Unable to initialize scene");
        scenes.push(Box::new(scene));
    }

    app.run(scenes).expect("Failed to run application");
}
