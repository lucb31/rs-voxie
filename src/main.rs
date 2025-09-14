use std::env;

use application::Application;
use scene::Scene;

mod application;
mod benchmark;
mod camera;
mod cube;
mod objmesh;
mod quadmesh;
mod scene;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let benchmark_enabled =
        args.contains(&"--benchmark".to_string()) || args.contains(&"-b".to_string());

    // Setup application
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let gl_ctx = app.gl_context().clone();

    // Setup scene(s) to render
    let mut scenes: Vec<Scene> = vec![];
    if benchmark_enabled {
        println!("Running in benchmark mode...");
        app.max_scene_duration_secs = 2.0;
        for i in 8..13 {
            let base: usize = 2;
            let count = base.pow(i);
            let mut scene = scene::Scene::new(&gl_ctx).expect("Unable to initialize scene");
            scene
                .add_cubes(&gl_ctx, count)
                .expect("Unable to init cubes");
            scene.title = format!("{count} cubes");
            scenes.push(scene);
        }
        // Start with the easy scenes
        scenes.reverse();
    } else {
        println!("Running game...");
        let mut scene = scene::Scene::new(&gl_ctx).expect("Unable to initialize scene");
        scene.add_cubes(&gl_ctx, 4).expect("Unable to init cubes");
        scene.title = "Game".to_string();
        scenes.push(scene);
    }

    app.run(scenes).expect("Failed to run application");
}
