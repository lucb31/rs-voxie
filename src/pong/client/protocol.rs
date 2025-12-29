use std::time::{Duration, Instant};

use log::debug;

use crate::{
    log_err,
    network::NetworkClient,
    pong::network::{NetworkCodec, NetworkCommand},
    scenes::metrics::SimpleMovingAverage,
};

use std::sync::mpsc::Receiver;

/// Networking protocol layer which handles conversion of game-specific commands & messages into
/// format that transport layer expects
pub struct ClientProtocol<C: NetworkCodec> {
    codec: std::marker::PhantomData<C>,
    downstream_bytes_rx: Receiver<Vec<u8>>,
    client: NetworkClient,

    initialized_at: Instant,
    last_ping: Instant,
    sma_ping: SimpleMovingAverage,
    health: bool,
}

impl<C: NetworkCodec> ClientProtocol<C> {
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
            codec: std::marker::PhantomData,
            initialized_at,
            downstream_bytes_rx,
            last_ping: Instant::now(),
        })
    }

    pub fn try_recv(&mut self) -> Option<NetworkCommand> {
        while let Ok(bytes) = self.downstream_bytes_rx.try_recv() {
            match C::decode(&bytes) {
                Ok(cmd) => match cmd {
                    NetworkCommand::ServerPong { timestamp } => {
                        let recv_time = self.initialized_at.elapsed().as_nanos();
                        let delta = recv_time - timestamp;
                        self.sma_ping.add(delta as f32);
                        self.health = true;
                    }
                    _ => return Some(cmd),
                },
                Err(e) => eprintln!("Decode error: {e}"),
            }
        }
        None
    }

    pub fn tick(&mut self) {
        // Ping once a second
        if self.last_ping.elapsed() > Duration::from_secs(1) {
            log_err!(
                self.send_cmd(NetworkCommand::ClientPing {
                    timestamp: self.initialized_at.elapsed().as_nanos(),
                }),
                "Could not ping: {err}"
            );
            self.last_ping = Instant::now();
        }
    }

    pub fn send_cmd(&mut self, cmd: NetworkCommand) -> Result<(), String> {
        debug!("Sending command: {cmd:?}");
        let encoded = C::encode(&cmd).or(Err("Failed encoding".to_string()))?;
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
            });
    }
}
