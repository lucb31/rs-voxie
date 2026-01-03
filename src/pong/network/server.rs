use serde::{Deserialize, Serialize};

use crate::network::{EntitySnapshot, NetEntityId};

#[derive(Debug, Serialize, Deserialize)]
pub enum ServerMessage {
    SendSnapshot {
        server_tick: u32,
        last_acked_client_tick: u32,
        data: Vec<EntitySnapshot>,
    },
    StartRound {
        ball_net_entity: NetEntityId,
        frame: u32,
    },
    SpawnPlayer {
        player_net_entity: NetEntityId,
        player_slot: usize,
    },
    SpawnPaddle {
        net_entity_id: NetEntityId,
        player_slot: usize,
    },
    EndRound {
        loosing_player_slot: usize,
    },
    DespawnEntity {
        net_entity_id: NetEntityId,
    },
}
