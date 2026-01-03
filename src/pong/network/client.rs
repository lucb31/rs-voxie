use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::network::NetEntityId;

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    RequestJoin,
    InputSync {
        last_acked_client_tick: u32,
        unacked_inputs: Vec<InputSample>,
    },
    UpdatePlayerInputVelocity {
        net_entity_id: NetEntityId,
        input_velocity: Vec3,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSample {
    pub(crate) client_tick: u32,
    pub(crate) vertical_velocity: f32,
}

#[cfg(test)]
mod tests {
    use glam::Vec3;

    use crate::pong::network::client::ClientMessage;

    #[test]
    fn encode_decode_equals() {
        let cmd = ClientMessage::UpdatePlayerInputVelocity {
            net_entity_id: 0,
            input_velocity: Vec3::new(1.0, 2.0, 3.0),
        };
        let encoded = bincode::serialize(&cmd).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        println!("{decoded:?}");
        assert!(
            matches!(
                decoded,
                ClientMessage::UpdatePlayerInputVelocity {
                    net_entity_id,
                    input_velocity
                }
            ),
            "Decoded message does not equal original message"
        );
    }
}
