use log::{debug, error};

use crate::{
    network::{ClientId, NetworkServer, ServerDownstreamPayload, ServerUpstreamPayload},
    pong::network::{NetworkCodec, ServerMessage, client::ClientMessage},
};

use std::sync::mpsc::Receiver;

/// Networking protocol layer which handles conversion of game-specific commands & messages into
/// format that transport layer expects
pub struct ServerProtocol<C: NetworkCodec> {
    codec: std::marker::PhantomData<C>,
    upstream_payload_rx: Receiver<ServerUpstreamPayload>,

    server: NetworkServer,
}

impl<C: NetworkCodec> ServerProtocol<C> {
    pub fn new(
        server: NetworkServer,
        upstream_payload_rx: Receiver<ServerUpstreamPayload>,
    ) -> Result<Self, String> {
        Ok(ServerProtocol {
            server,
            codec: std::marker::PhantomData,
            upstream_payload_rx,
        })
    }

    /// Decode incoming bytes from transport layer
    pub fn try_recv(&mut self) -> Option<ClientMessage> {
        while let Ok(payload) = self.upstream_payload_rx.try_recv() {
            match serde_json::from_str(&String::from_utf8_lossy(&payload.bytes)) {
                Ok(cmd) => return Some(cmd),
                Err(e) => eprintln!("Decode error: {e}"),
            }
        }
        None
    }

    fn send_to(&self, cmd: ServerMessage, client: ClientId) {
        match C::encode(&cmd) {
            Ok(bytes) => self
                .server
                .send(ServerDownstreamPayload::new(bytes, Some(client)))
                .expect("Unable to send"),
            Err(_) => error!("Unable to encode cmd "),
        }
    }

    pub fn broadcast(&self, cmd: ServerMessage) -> Result<(), String> {
        let encoded = C::encode(&cmd).or(Err("Failed encoding".to_string()))?;
        self.server
            .send(ServerDownstreamPayload::new(encoded, None))
            .or(Err("Unable to send".to_string()))
    }
}
