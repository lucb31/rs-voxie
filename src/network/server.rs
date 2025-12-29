use std::{
    collections::HashSet,
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};

use log::{debug, error, info};

pub type ClientId = SocketAddr;

pub struct ServerDownstreamPayload {
    bytes: Vec<u8>,
    client: Option<ClientId>,
}

impl ServerDownstreamPayload {
    pub fn new(bytes: Vec<u8>, client: Option<ClientId>) -> ServerDownstreamPayload {
        Self { bytes, client }
    }
}

pub struct ServerUpstreamPayload {
    pub bytes: Vec<u8>,
    pub client: ClientId,
}

impl ServerUpstreamPayload {
    pub fn new(bytes: Vec<u8>, client: ClientId) -> ServerUpstreamPayload {
        Self { bytes, client }
    }
}

/// Transport layer for server-client communication
pub struct NetworkServer {
    // WARN: Need better connection management. At least should timeout if we didnt hear from a
    // client for X seconds
    connected_clients: Arc<Mutex<HashSet<ClientId>>>,
    downstream_tx: Option<Sender<ServerDownstreamPayload>>,
}

impl NetworkServer {
    pub fn new() -> Self {
        Self {
            connected_clients: Arc::new(Mutex::new(HashSet::new())),
            downstream_tx: None,
        }
    }

    pub fn send(&self, payload: ServerDownstreamPayload) -> Result<(), String> {
        debug_assert!(
            self.downstream_tx.is_some(),
            "Send called before serve. Not allowed"
        );
        self.downstream_tx
            .as_ref()
            .unwrap()
            .send(payload)
            .map_err(|_| "Failed to send bytes".to_string())
    }

    /// Start communication thread. Non-blocking
    pub fn serve(&mut self, upstream_tx: Sender<ServerUpstreamPayload>) -> std::io::Result<()> {
        let server_address = "127.0.0.1:8080";
        let socket = UdpSocket::bind(server_address)?;
        socket.set_nonblocking(true)?;
        info!("Server listening at {server_address}");

        // Communication thread
        let clients = Arc::clone(&self.connected_clients);
        let (downstream_tx, downstream_rx) = mpsc::channel::<ServerDownstreamPayload>();
        self.downstream_tx = Some(downstream_tx);
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                // Encode & Send queued downstream packets
                while let Ok(payload) = downstream_rx.try_recv() {
                    match payload.client {
                        Some(client) => {
                            debug!(
                                "Sending message {} to single client {}",
                                String::from_utf8_lossy(&payload.bytes),
                                client
                            );
                            socket.send_to(&payload.bytes, client).unwrap();
                        }
                        None => {
                            debug!(
                                "Broadcasting message {}",
                                String::from_utf8_lossy(&payload.bytes)
                            );
                            for client in clients.lock().unwrap().iter() {
                                socket.send_to(&payload.bytes, client).unwrap();
                            }
                        }
                    }
                }

                // Read network packages: Client -> Server = upstream communication
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, client_address)) => {
                            let payload = &buf[..n];
                            // TODO: Better connection management
                            clients.lock().unwrap().insert(client_address);
                            upstream_tx
                                .send(ServerUpstreamPayload::new(payload.to_vec(), client_address))
                                .expect("Could not send upstream");
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

        Ok(())
    }
}
