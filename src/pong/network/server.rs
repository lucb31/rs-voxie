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
        ai_net_entity: NetEntityId,
        player_net_entity: NetEntityId,
    },
    EndRound {
        winner: u32,
    },
    DespawnEntity {
        net_entity_id: NetEntityId,
    },
}
