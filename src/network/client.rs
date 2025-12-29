use std::{
    error::Error,
    net::UdpSocket,
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    thread,
    time::Duration,
};

use log::{debug, error, info};

use super::meter::TrafficMeter;

/// Networking transport layer. Manages UDP connection
/// Needs to be enhanced with game specific protocol layer
pub struct NetworkClient {
    // Communcation channel to send messages to the server
    upstream_tx: Sender<Vec<u8>>,
    traffic_meter: Arc<Mutex<TrafficMeter>>,
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
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                // Send queued messages
                while let Ok(msg) = upstream_rx.try_recv() {
                    debug!("Sending: {}", String::from_utf8_lossy(&msg));
                    if let Err(e) = socket.send(&msg) {
                        error!("Error sending message from client to server: {e}");
                    } else if let Ok(mut meter) = thread_meter.lock() {
                        meter.track_upstream(msg.len());
                    }
                }

                // Receive incoming packets
                loop {
                    match socket.recv(&mut buf) {
                        Ok(n) => {
                            let payload = &buf[..n];
                            debug!("Receiving: {}", String::from_utf8_lossy(payload));
                            if let Err(e) = downstream_tx.send(payload.to_vec()) {
                                error!("Failed to forward payload to protocol layer: {e}");
                            } else if let Ok(mut meter) = thread_meter.lock() {
                                meter.track_downstream(payload.len());
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
            upstream_tx,
            traffic_meter,
        })
    }

    pub fn downstream_bps(&self) -> u64 {
        self.traffic_meter.lock().unwrap().downstream_bps()
    }

    pub fn upstream_bps(&self) -> u64 {
        self.traffic_meter.lock().unwrap().upstream_bps()
    }

    pub fn send_bytes(&self, bytes: Vec<u8>) -> Result<(), String> {
        self.upstream_tx
            .send(bytes)
            .map_err(|_| "Failed to send bytes".to_string())
    }
}
