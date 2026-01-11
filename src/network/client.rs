use std::{
    error::Error,
    net::UdpSocket,
    sync::{
        Arc, Mutex, RwLock,
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use log::{debug, error, info};

use crate::{network::message::NetworkMessage, util::SimpleMovingAverage};

use super::{ClientId, meter::TrafficMeter};

/// Networking transport layer. Manages UDP connection
/// Needs to be enhanced with game specific protocol layer
pub struct NetworkClient {
    // Communcation channel to send messages to the server
    upstream_tx: Sender<Vec<u8>>,
    traffic_meter: Arc<Mutex<TrafficMeter>>,
    socket: UdpSocket,

    client_id: Arc<RwLock<Option<ClientId>>>,

    // Ping information
    initialized_at: Instant,
    ping_sma: Arc<RwLock<SimpleMovingAverage>>,
}

impl NetworkClient {
    pub fn new(
        server_address: &str,
        // Channel to pass incoming bytes to protocol layer
        downstream_tx: Sender<Vec<u8>>,
    ) -> Result<NetworkClient, Box<dyn Error>> {
        // Bind to 0.0.0.0:0 to let OS pick an available port
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(server_address)?;
        socket.set_nonblocking(true)?;
        info!(
            "Initialized client at {}, connected to {server_address}",
            socket.local_addr()?
        );

        // Spawn transport thread
        let (upstream_tx, upstream_rx) = mpsc::channel::<Vec<u8>>();
        let traffic_meter = Arc::new(Mutex::new(TrafficMeter::new()));
        let thread_meter = Arc::clone(&traffic_meter);

        let socket_clone = socket.try_clone().unwrap();
        let ping_sma = Arc::new(RwLock::new(SimpleMovingAverage::new(10)));
        let ping_sma_thread = Arc::clone(&ping_sma);
        let initialized_at = Instant::now();
        let initialized_at_thread = initialized_at;
        let client_id = Arc::new(RwLock::new(None));
        let client_id_thread = Arc::clone(&client_id);
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                // Send queued messages
                while let Ok(packet) = upstream_rx.try_recv() {
                    // Convert to network message
                    match bincode::serialize(&NetworkMessage::GamePacket { payload: packet }) {
                        Ok(msg) => {
                            if let Err(e) = socket.send(&msg) {
                                error!("Error sending message from client to server: {e}");
                            } else if let Ok(mut meter) = thread_meter.lock() {
                                meter.track_upstream(msg.len());
                            }
                        }
                        Err(err) => error!("Failed to serialize packet: {err}"),
                    }
                }

                // Receive incoming packets
                loop {
                    match socket.recv(&mut buf) {
                        Ok(n) => {
                            let payload = &buf[..n];
                            // Deserialize to network message
                            // Special cases for non-game packets
                            match bincode::deserialize::<NetworkMessage>(payload) {
                                Ok(network_msg) => match network_msg {
                                    NetworkMessage::Ping { .. } => {
                                        error!("Client received ping, this should not happen");
                                    }
                                    NetworkMessage::Pong {
                                        client_id,
                                        client_timestamp,
                                        server_uptime,
                                    } => {
                                        debug!(
                                            "Received pong {client_id}, {client_timestamp}, {server_uptime}"
                                        );
                                        let recv_time = initialized_at_thread.elapsed().as_nanos();
                                        let delta = recv_time - client_timestamp;
                                        ping_sma_thread.write().unwrap().add(delta as f32);
                                        *client_id_thread.write().unwrap() = Some(client_id);
                                    }
                                    NetworkMessage::GamePacket { payload } => {
                                        let size = payload.len();
                                        if let Err(e) = downstream_tx.send(payload) {
                                            error!(
                                                "Failed to forward payload to protocol layer: {e}"
                                            );
                                        } else if let Ok(mut meter) = thread_meter.lock() {
                                            meter.track_downstream(size);
                                        }
                                    }
                                },
                                Err(err) => error!("Failed to deserialize network payload: {err}"),
                            }
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
                // Throttle CPU: sleep one frame (adjust to tick rate)
                thread::sleep(Duration::from_millis(1));
            }
        });

        Ok(NetworkClient {
            client_id,
            ping_sma,
            socket: socket_clone,
            upstream_tx,
            traffic_meter,
            initialized_at: Instant::now(),
        })
    }

    pub fn get_client_id(&self) -> Option<ClientId> {
        self.client_id.read().ok().and_then(|g| *g)
    }

    pub fn downstream_bps(&self) -> u64 {
        self.traffic_meter.lock().unwrap().downstream_bps()
    }

    pub fn upstream_bps(&self) -> u64 {
        self.traffic_meter.lock().unwrap().upstream_bps()
    }

    pub fn ping(&self) {
        let ping = NetworkMessage::Ping {
            client_timestamp: self.initialized_at.elapsed().as_nanos(),
        };
        match bincode::serialize(&ping) {
            Ok(bytes) => {
                if let Err(err) = self.socket.send(&bytes) {
                    error!("Failed to send ping: {err}");
                }
            }
            Err(err) => error!("Failed to serialize ping: {err}"),
        }
    }

    pub fn get_ping(&self) -> f32 {
        self.ping_sma.read().unwrap().get()
    }

    pub fn send_game_packet(&self, bytes: Vec<u8>) -> Result<(), String> {
        self.upstream_tx
            .send(bytes)
            .map_err(|_| "Failed to send bytes".to_string())
    }
}
