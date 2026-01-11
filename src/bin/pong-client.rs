use std::sync::mpsc;

use rs_voxie::{
    application::Application,
    network::NetworkClient,
    pong::{ClientProtocol, client::scene::PongScene},
};

fn main() {
    // Config setup
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let server_address = std::env::var("SERVER_ADDRESS").unwrap_or("127.0.0.1:7777".to_string());

    // NETWORKING
    // Setup transport layer
    let (downstream_bytes_tx, downstream_bytes_rx) = mpsc::channel::<Vec<u8>>();
    let client = NetworkClient::new(&server_address, downstream_bytes_tx)
        .expect("Could not initialize transport layer");
    // Setup protocol layer
    let protocol =
        ClientProtocol::new(downstream_bytes_rx, client).expect("Could not init client proto");

    // Setup scene
    let mut app = Application::new("Voxie").expect("Could not setup application");
    let scene =
        PongScene::new(protocol, app.input_state.clone()).expect("Could not init pong scene");
    app.add_scene(Box::new(scene));

    app.run().expect("Failed to run application");
}
