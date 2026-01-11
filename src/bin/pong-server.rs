use std::sync::mpsc;

use rs_voxie::network::{HeadlessSimulation, NetworkServer, ServerUpstreamPayload};
use rs_voxie::pong::server::scene::PongServerScene;
use rs_voxie::pong::{BincodeCodec, ServerProtocol};

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Setup transport layer
    let mut server = NetworkServer::new();
    let (upstream_tx, upstream_rx) = mpsc::channel::<ServerUpstreamPayload>();
    server
        .serve("0.0.0.0:7777", upstream_tx)
        .expect("Could not serve");

    // Setup protocol layer
    let protocol =
        ServerProtocol::<BincodeCodec>::new(server, upstream_rx).expect("Could not init protocol");

    let scene = PongServerScene::new(protocol).expect("Could not initialize pong scene");
    let mut simulation = HeadlessSimulation::new(Box::new(scene));
    simulation.run();
}
