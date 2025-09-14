use std::env;

use application::Application;

mod application;
mod camera;
mod cube;
mod objmesh;
mod quadmesh;
mod scene;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <nr of cubes>", args[0]);
        std::process::exit(1);
    }
    let cube_count = &args[1].parse::<usize>().expect("Not a valid number");
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let gl_ctx = app.gl_context();

    let mut scene = scene::Scene::new(gl_ctx).expect("Unable to initialize scene");
    scene.add_cubes(gl_ctx, *cube_count);

    app.scene = Some(scene);

    app.run().expect("Failed to run application");
}
