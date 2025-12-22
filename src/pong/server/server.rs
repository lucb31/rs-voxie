use std::{
    collections::HashSet,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

use log::{debug, error, info, warn};

use crate::network::{JsonCodec, NetworkCodec, NetworkCommand};

use super::simulation::PongSimulation;

pub struct PongServer<C: NetworkCodec> {
    codec: std::marker::PhantomData<C>,
    connected_clients: Arc<Mutex<HashSet<SocketAddr>>>,
}

impl<C: NetworkCodec> PongServer<C> {
    pub fn new() -> PongServer<C> {
        Self {
            codec: std::marker::PhantomData,
            connected_clients: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn serve(&mut self) -> std::io::Result<()> {
        let server_address = "127.0.0.1:8080";
        let socket = UdpSocket::bind(server_address)?;
        socket.set_nonblocking(true)?;
        info!("Server listening at {server_address}");

        // Communication channel for messages to all connected clients
        let (broadcast_tx, broadcast_rx) = mpsc::channel::<Vec<u8>>();
        let (simulation_start_tx, simulation_start_rx) = mpsc::channel::<SocketAddr>();
        let clients = Arc::clone(&self.connected_clients);
        // Communication thread
        let communication_handle = thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                // Send queued messages
                while let Ok(msg) = broadcast_rx.try_recv() {
                    debug!("Broadcasting message {}", String::from_utf8_lossy(&msg));
                    for client in clients.lock().unwrap().iter() {
                        debug!("Sending: {} to {}", String::from_utf8_lossy(&msg), client);
                        socket.send_to(&msg, client).unwrap();
                    }
                }

                // read all available packets
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, client_address)) => {
                            let payload = &buf[..n];
                            match C::decode(payload) {
                                Ok(cmd) => match cmd {
                                    crate::network::NetworkCommand::ClientStartRound => {
                                        // BUG: Don't allow start while sim in progress
                                        debug!("Received start command");
                                        clients.lock().unwrap().insert(client_address);
                                        // Once start command received, start the simulation
                                        simulation_start_tx
                                            .send(client_address)
                                            .expect("Could not send");
                                    }
                                    crate::network::NetworkCommand::ClientPing { timestamp } => {
                                        debug!("Received ping command");
                                        // Respond to ping right away
                                        let cmd = NetworkCommand::ServerPong { timestamp };
                                        let encoded = C::encode(&cmd).unwrap();
                                        socket
                                            .send_to(&encoded, client_address)
                                            .expect("Could not reply to client");
                                    }
                                    _ => {
                                        error!(
                                            "Server does not know how to handle this command: {cmd:?}"
                                        );
                                    }
                                },
                                Err(err) => error!("Could not decode network payload: {err}"),
                            }
                        }

                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No packets remaining this tick
                            break;
                        }

                        Err(e) => {
                            error!("socket error: {:?}", e);
                            break;
                        }
                    }
                }
                // Throttle CPU: sleep one frame (adjust to tick rate)
                thread::sleep(Duration::from_millis(1));
            }
        });

        // simulation thread
        let simulation_handle = thread::spawn(move || {
            loop {
                match simulation_start_rx.recv() {
                    Ok(client_address) => {
                        info!("Client connected {client_address}. Let's go!");
                        // TODO: The codec should be on client level, not on sim
                        // Instead of passing the broadcast tx we should pass a server
                        // network server instance
                        let mut simulation = PongSimulation::<JsonCodec>::new();
                        simulation.run(broadcast_tx.clone());
                        info!("Simulation done. Waiting for new connection.");
                    }
                    Err(err) => error!("Could not receive start signal{err}"),
                }
            }
        });

        simulation_handle.join().unwrap();
        communication_handle.join().unwrap();

        Ok(())
    }
}
