use std::{
    collections::HashMap,
    net::{SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use log::{debug, error, info, trace};

use crate::{log_err, network::message::NetworkMessage};

/// Interval in which the server checks for inactive clients
const INACTIVE_CLIENT_CHECK_INTERVAL: Duration = Duration::from_secs(1);
/// Duration elapsed since the last successful ping to an active client for it to be considered
/// inactive
const INACTIVE_CLIENT_TIMEOUT_DURATION: Duration = Duration::from_secs(3);

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

#[derive(Debug)]
pub enum ServerEvent {
    ClientConnected(ClientId),
    ClientDisconnected(ClientId),
}

struct ClientInfo {
    last_ping_received: Instant,
}

/// Transport layer for server-client communication
pub struct NetworkServer {
    connected_clients: Arc<Mutex<HashMap<ClientId, ClientInfo>>>,
    downstream_tx: Option<Sender<ServerDownstreamPayload>>,
    event_rx: Option<Receiver<ServerEvent>>,
}

impl NetworkServer {
    pub fn new() -> Self {
        Self {
            connected_clients: Arc::new(Mutex::new(HashMap::new())),
            downstream_tx: None,
            event_rx: None,
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

    pub fn try_recv_event(&mut self) -> Option<ServerEvent> {
        if let Ok(event) = self.event_rx.as_mut()?.try_recv() {
            info!("Received server event: {event:?}");
            return Some(event);
        }
        None
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
        let (event_tx, event_rx) = mpsc::channel::<ServerEvent>();
        self.downstream_tx = Some(downstream_tx);
        let upstream_tx_thread = upstream_tx.clone();
        self.event_rx = Some(event_rx);
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            let mut last_inactive_client_check_at = Instant::now();
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
                                for (client_id, ..) in clients.lock().unwrap().iter() {
                                    socket.send_to(&bytes, client_id).unwrap();
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
                                    &upstream_tx_thread,
                                    &event_tx
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

                // Check for inactive clients
                if last_inactive_client_check_at.elapsed() > INACTIVE_CLIENT_CHECK_INTERVAL {
                    let mut clients_mutex = clients.lock().unwrap();
                    let mut inactive_clients: Vec<ClientId> = vec![];
                    for (k, v) in clients_mutex.iter() {
                        if v.last_ping_received.elapsed() > INACTIVE_CLIENT_TIMEOUT_DURATION {
                            inactive_clients.push(*k);
                        }
                    }
                    for client in inactive_clients {
                        debug!("Removing inactive client {client}");
                        clients_mutex.remove(&client);
                        event_tx
                            .send(ServerEvent::ClientDisconnected(client))
                            .expect("Unable to send disconnect event");
                    }
                    last_inactive_client_check_at = Instant::now();
                }

                // Throttle CPU
                thread::sleep(Duration::from_millis(1));
            }
        });

        Ok(())
    }
}

impl Default for NetworkServer {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper layer around network packets to separate concerns of
/// - Network packets such as ping-pong and
/// - Game packets -> Handed to channel and game implementation to process
fn process_received_bytes(
    socket: &UdpSocket,
    initialized_at: &Instant,
    payload: &[u8],
    clients: &Arc<Mutex<HashMap<ClientId, ClientInfo>>>,
    client_address: SocketAddr,
    upstream_tx: &Sender<ServerUpstreamPayload>,
    server_event_tx: &Sender<ServerEvent>,
) -> Result<(), String> {
    let network_message: NetworkMessage = bincode::deserialize(payload)
        .map_err(|err| format!("Failed to decode into NetworkMessage: {err}"))?;
    match network_message {
        NetworkMessage::Ping { client_timestamp } => {
            {
                let mut lock = clients.lock().unwrap();
                match lock.get_mut(&client_address) {
                    Some(info) => info.last_ping_received = Instant::now(),
                    None => {
                        lock.insert(
                            client_address,
                            ClientInfo {
                                last_ping_received: Instant::now(),
                            },
                        );
                        server_event_tx
                            .send(ServerEvent::ClientConnected(client_address))
                            .expect("Unable to send client connect event");
                    }
                }
            }
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
