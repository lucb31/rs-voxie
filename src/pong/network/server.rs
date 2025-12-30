use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{network::NetEntityId, systems::physics::Transform};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "data")]
pub enum ServerMessage {
    Pong {
        timestamp: u128,
    },
    UpdateTransform {
        net_entity_id: NetEntityId,
        transform: Transform,
    },
    StartRound {
        ball_net_entity: NetEntityId,
    },
    SpawnPlayer {
        player_net_entity: NetEntityId,
        position: Vec3,
    },
    SpawnPaddle {
        net_entity_id: NetEntityId,
        position: Vec3,
    },
    EndRound {
        winner: u32,
    },
    DespawnEntity {
        net_entity_id: NetEntityId,
    },
}
