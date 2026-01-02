use serde::{Deserialize, Serialize};

use super::ClientId;

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum NetworkMessage {
    Ping {
        client_timestamp: u128,
    },
    Pong {
        client_id: ClientId,
        client_timestamp: u128,
        server_uptime: u128,
    },
    GamePacket {
        payload: Vec<u8>,
    },
}
