use std::time::{Duration, Instant};

use log::{debug, error};

use crate::{
    log_err,
    network::NetworkClient,
    pong::network::{ServerMessage, client::ClientMessage},
    scenes::metrics::SimpleMovingAverage,
};

use std::sync::mpsc::Receiver;

/// Networking protocol layer which handles conversion of game-specific commands & messages into
/// format that transport layer expects
pub struct ClientProtocol {
    downstream_bytes_rx: Receiver<Vec<u8>>,
    client: NetworkClient,

    initialized_at: Instant,
    last_ping: Instant,
    sma_ping: SimpleMovingAverage,
    health: bool,
}

impl ClientProtocol {
    pub fn new(
        downstream_bytes_rx: Receiver<Vec<u8>>,
        client: NetworkClient,
    ) -> Result<Self, String> {
        let sma_ping = SimpleMovingAverage::new(5);
        let health = false;

        let initialized_at = Instant::now();
        Ok(ClientProtocol {
            client,
            sma_ping,
            health,
            initialized_at,
            downstream_bytes_rx,
            last_ping: Instant::now(),
        })
    }

    pub fn get_rtt_estimate(&self) -> Duration {
        Duration::from_nanos(self.sma_ping.get() as u64)
    }

    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        while let Ok(bytes) = self.downstream_bytes_rx.try_recv() {
            match bincode::deserialize(&bytes) {
                Ok(cmd) => match cmd {
                    ServerMessage::Pong { timestamp } => {
                        let recv_time = self.initialized_at.elapsed().as_nanos();
                        let delta = recv_time - timestamp;
                        self.sma_ping.add(delta as f32);
                        self.health = true;
                    }
                    _ => return Some(cmd),
                },
                Err(e) => error!("Decode error: {e}"),
            }
        }
        None
    }

    pub fn tick(&mut self) {
        // Ping once a second
        if self.last_ping.elapsed() > Duration::from_secs(1) {
            log_err!(
                self.send_cmd(ClientMessage::Ping {
                    timestamp: self.initialized_at.elapsed().as_nanos(),
                }),
                "Could not ping: {err}"
            );
            self.last_ping = Instant::now();
        }
    }

    pub fn send_cmd(&self, cmd: ClientMessage) -> Result<(), String> {
        debug!("Sending command: {cmd:?}");
        let encoded = bincode::serialize(&cmd).or(Err("Failed encoding".to_string()))?;
        log_err!(
            self.client
                .send_bytes(encoded)
                .or(Err("Error sending".to_string())),
            "Could not send client cmd {cmd:?}: {err}"
        );
        Ok(())
    }

    pub fn render_ui(&self, ui: &mut imgui::Ui) {
        ui.window("Network")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([500.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Health: {}", self.health));
                ui.text(format!("Ping: {:.1}ms", self.sma_ping.get() * 1e-6,));
                ui.text(format!(
                    "Upstream: {:.1}kbps",
                    self.client.upstream_bps() as f32 * 1e-3
                ));
                ui.text(format!(
                    "Downstream: {:.1}kbps",
                    self.client.downstream_bps() as f32 * 1e-3
                ));
            });
    }
}
