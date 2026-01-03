use std::time::{Duration, Instant};

use log::{error, trace};

use crate::{
    network::{ClientId, NetworkClient},
    pong::network::{ServerMessage, client::ClientMessage},
};

use std::sync::mpsc::Receiver;

/// Networking protocol layer which handles conversion of game-specific commands & messages into
/// format that transport layer expects
pub struct ClientProtocol {
    downstream_bytes_rx: Receiver<Vec<u8>>,
    client: NetworkClient,
    last_ping: Instant,
    client_tick: u32,
}

impl ClientProtocol {
    pub fn new(
        downstream_bytes_rx: Receiver<Vec<u8>>,
        client: NetworkClient,
    ) -> Result<Self, String> {
        Ok(ClientProtocol {
            client,
            downstream_bytes_rx,
            last_ping: Instant::now(),
            client_tick: 0,
        })
    }

    pub fn get_client_id(&self) -> Option<ClientId> {
        self.client.get_client_id()
    }

    pub fn get_client_tick(&self) -> u32 {
        self.client_tick
    }

    pub fn get_rtt_estimate(&self) -> Duration {
        Duration::from_nanos(self.client.get_ping() as u64)
    }

    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        while let Ok(bytes) = self.downstream_bytes_rx.try_recv() {
            match bincode::deserialize(&bytes) {
                Ok(cmd) => return Some(cmd),
                Err(e) => error!("Decode error: {e}"),
            }
        }
        None
    }

    pub fn tick(&mut self) {
        // Ping once a second
        if self.last_ping.elapsed() > Duration::from_secs(1) {
            self.client.ping();
            self.last_ping = Instant::now();
        }
        self.client_tick += 1;
    }

    pub fn send_cmd(&self, cmd: ClientMessage) -> Result<(), String> {
        trace!("Sending command: {cmd:?}");
        let encoded = bincode::serialize(&cmd).or(Err("Failed encoding".to_string()))?;
        self.client
            .send_game_packet(encoded)
            .or(Err("Error sending: {cmd:?}".to_string()))
    }

    pub fn render_ui(&self, ui: &mut imgui::Ui) {
        ui.window("Network")
            .size([150.0, 100.0], imgui::Condition::FirstUseEver)
            .position([500.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("ClientId: {:?}", self.get_client_id()));
                ui.text(format!("Ping: {:.1}ms", self.client.get_ping() * 1e-6,));
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
