use serde::{Deserialize, Serialize};

use crate::{network::NetEntityId, systems::physics::Transform};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "data")]
pub enum ServerMessage {
    ServerPong {
        timestamp: u128,
    },
    ServerUpdateTransform {
        net_entity_id: NetEntityId,
        transform: Transform,
    },
    ServerStartRound {
        ball_net_entity: NetEntityId,
        ai_net_entity: NetEntityId,
    },
    ServerEndRound {
        winner: u32,
    },
    ServerDespawnEntity {
        net_entity_id: NetEntityId,
    },
}
