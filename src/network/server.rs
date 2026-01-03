use std::{
    collections::HashSet,
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use log::{error, info, trace};

use crate::{log_err, network::message::NetworkMessage};

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

    pub fn send_game_packet(&self, payload: ServerDownstreamPayload) -> Result<(), String> {
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
    pub fn serve(
        &mut self,
        server_address: &str,
        upstream_tx: Sender<ServerUpstreamPayload>,
    ) -> std::io::Result<()> {
        let socket = UdpSocket::bind(server_address)?;
        socket.set_nonblocking(true)?;
        info!("Server listening at {server_address}");

        // Communication thread
        let clients = Arc::clone(&self.connected_clients);
        let initialized_at = Instant::now();
        let (downstream_tx, downstream_rx) = mpsc::channel::<ServerDownstreamPayload>();
        self.downstream_tx = Some(downstream_tx);
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                // Encode & Send queued downstream game packets
                while let Ok(payload) = downstream_rx.try_recv() {
                    // Wrap into network message
                    let packet = NetworkMessage::GamePacket {
                        payload: payload.bytes,
                    };
                    match bincode::serialize(&packet) {
                        Ok(bytes) => match payload.client {
                            Some(client) => {
                                trace!("Sending message to single client {client}");
                                socket.send_to(&bytes, client).unwrap();
                            }
                            None => {
                                trace!("Broadcasting message");
                                for client in clients.lock().unwrap().iter() {
                                    socket.send_to(&bytes, client).unwrap();
                                }
                            }
                        },
                        Err(err) => {
                            error!("Failed to serialize game packet: {err}");
                            continue;
                        }
                    }
                }

                // Upstream communication: Packets that a client has sent to the server
                // Read network packages: Client -> Server = upstream communication
                loop {
                    match socket.recv_from(&mut buf) {
                        Ok((n, client_address)) => {
                            let payload = &buf[..n];
                            log_err!(
                                process_received_bytes(
                                    &socket,
                                    &initialized_at,
                                    payload,
                                    &clients,
                                    client_address,
                                    upstream_tx.clone(),
                                ),
                                "Failed to process received bytes: {err}"
                            );
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            // No packets remaining this tick
                            break;
                        }
                        Err(e) => {
                            error!("socket error: {e:?}");
                            break;
                        }
                    }
                }
                // Throttle CPU
                thread::sleep(Duration::from_millis(1));
            }
        });

        Ok(())
    }
}

/// Wrapper layer around network packets to separate concerns of
/// - Network packets such as ping-pong and
/// - Game packets -> Handed to channel and game implementation to process
fn process_received_bytes(
    socket: &UdpSocket,
    initialized_at: &Instant,
    payload: &[u8],
    clients: &Arc<Mutex<HashSet<ClientId>>>,
    client_address: SocketAddr,
    upstream_tx: Sender<ServerUpstreamPayload>,
) -> Result<(), String> {
    let network_message: NetworkMessage = bincode::deserialize(payload)
        .map_err(|err| format!("Failed to decode into NetworkMessage: {err}"))?;
    match network_message {
        NetworkMessage::Ping { client_timestamp } => {
            clients.lock().unwrap().insert(client_address);
            let response = NetworkMessage::Pong {
                client_id: client_address,
                client_timestamp,
                server_uptime: initialized_at.elapsed().as_nanos(),
            };
            let encoded = bincode::serialize(&response)
                .map_err(|err| format!("Unable to serialize pong: {err}"))?;
            socket
                .send_to(&encoded, client_address)
                .map_err(|err| format!("Unable to send pong: {err}"))?;
            Ok(())
        }
        NetworkMessage::Pong { .. } => {
            Err("Server received pong. This should never happen".to_string())
        }
        NetworkMessage::GamePacket { payload } => {
            // Game packets are handed to upstream channel
            upstream_tx
                .send(ServerUpstreamPayload::new(payload.to_vec(), client_address))
                .map_err(|err| format!("Unable to forward upstream payload: {err}"))?;
            Ok(())
        }
    }?;
    Ok(())
}
