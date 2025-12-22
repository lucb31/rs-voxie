use std::{
    error::Error,
    net::UdpSocket,
    sync::{
        Arc, RwLock,
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use log::{debug, error, info};

use crate::{
    network::NetEntityId, scenes::metrics::SimpleMovingAverage, systems::physics::Transform,
};

use super::{NetworkCodec, NetworkCommand};

pub struct NetworkClient<C: NetworkCodec> {
    codec: std::marker::PhantomData<C>,
    healthy: Arc<RwLock<bool>>,
    ping_sma: Arc<RwLock<SimpleMovingAverage>>,
    initialized_at: Instant,
    // Communcation channel to send messages to the server
    message_tx: Sender<Vec<u8>>,
}

impl<C: NetworkCodec> NetworkClient<C> {
    pub fn new(
        server_address: &str,
        cmd_buffer: Sender<NetworkCommand>,
    ) -> Result<NetworkClient<C>, Box<dyn Error>> {
        // Bind to 0.0.0.0:0 to let OS pick an available port
        let socket = UdpSocket::bind("0.0.0.0:0")?;
        socket.connect(server_address)?;
        socket.set_nonblocking(true)?;
        info!(
            "Initialized client at {}, connected to {server_address}",
            socket.local_addr()?
        );

        let sma = Arc::new(RwLock::new(SimpleMovingAverage::new(5)));
        let sma_ping_thread = Arc::clone(&sma);
        let health_lock = Arc::new(RwLock::new(false));
        let health_lock_pthread = Arc::clone(&health_lock);

        // Spawn communication thread
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let initialized_at = Instant::now();
        thread::spawn(move || {
            let mut buf = [0u8; 1024];
            loop {
                // Send queued messages
                while let Ok(msg) = rx.try_recv() {
                    debug!("Sent: {}", String::from_utf8_lossy(&msg));
                    socket.send(&msg).unwrap();
                }

                // read all available packets
                // Decode payloads to command
                // Pass Commands to command buffer
                loop {
                    match socket.recv(&mut buf) {
                        Ok(n) => {
                            let recv_time = initialized_at.elapsed().as_nanos();
                            let payload = &buf[..n];
                            match C::decode(payload) {
                                Ok(cmd) => {
                                    if let NetworkCommand::ServerPong { timestamp } = cmd {
                                        let delta = recv_time - timestamp;
                                        sma_ping_thread.write().unwrap().add(delta as f32);
                                        *health_lock_pthread.write().unwrap() = true;
                                    } else {
                                        if let Err(err) = cmd_buffer.send(cmd) {
                                            error!("Unable to command: {err}");
                                        }
                                    }
                                }
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

        Ok(NetworkClient {
            codec: std::marker::PhantomData,
            healthy: health_lock,
            ping_sma: sma,
            initialized_at,
            message_tx: tx,
        })
    }

    pub fn render_ui(&self, ui: &mut imgui::Ui) {
        ui.window("Network")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([500.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Health: {}", self.healthy.read().unwrap()));
                ui.text(format!(
                    "Ping: {:.1}ms",
                    self.ping_sma.read().unwrap().get() * 1e-6,
                ));
            });
    }

    pub fn send_cmd(&mut self, cmd: NetworkCommand) -> Result<(), String> {
        debug!("Sending command: {cmd:?}");
        let encoded = C::encode(&cmd).or(Err("Failed encoding".to_string()))?;
        self.message_tx
            .send(encoded)
            .or(Err("Error sending".to_string()));
        Ok(())
    }

    pub fn ping(&mut self) -> Result<(), String> {
        self.send_cmd(NetworkCommand::ClientPing {
            timestamp: self.initialized_at.elapsed().as_nanos(),
        })
    }
}
