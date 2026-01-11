use log::info;
use rs_voxie::{application::Application, voxie::scene::GameScene};

fn main() {
    // Config setup
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting voxie game scene...");

    // Setup scene
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let scene = GameScene::new(&app.gl_context().clone(), app.input_state.clone())
        .expect("Unable to init voxie scene");
    app.add_scene(Box::new(scene));

    app.run().expect("Failed to run application");
}
