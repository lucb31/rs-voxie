use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::systems::physics::Transform;

use super::NetEntityId;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "data")]
pub enum NetworkCommand {
    ClientStartRound,
    ClientPing {
        timestamp: u128,
    },
    ServerPong {
        timestamp: u128,
    },
    UpdateTransform {
        net_entity_id: NetEntityId,
        transform: Transform,
    },
    SpawnBall {
        net_entity_id: NetEntityId,
    },
    DespawnBall {
        net_entity_id: NetEntityId,
    },
}

pub trait NetworkCodec {
    type Error: Display + std::fmt::Debug;

    fn encode(cmd: &NetworkCommand) -> Result<Vec<u8>, Self::Error>;
    fn decode(input: &[u8]) -> Result<NetworkCommand, Self::Error>;
}

pub struct JsonCodec;

impl NetworkCodec for JsonCodec {
    type Error = serde_json::Error;

    fn encode(cmd: &NetworkCommand) -> Result<Vec<u8>, Self::Error> {
        Ok(serde_json::to_string(cmd)?.into())
    }

    fn decode(input: &[u8]) -> Result<NetworkCommand, Self::Error> {
        serde_json::from_str(&String::from_utf8_lossy(input))
    }
}
