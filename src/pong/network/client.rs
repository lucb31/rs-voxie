use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::network::NetEntityId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "data")]
pub enum ClientMessage {
    RequestJoin,
    UpdatePlayerInputVelocity {
        net_entity_id: NetEntityId,
        input_velocity: Vec3,
    },
    Ping {
        timestamp: u128,
    },
}
