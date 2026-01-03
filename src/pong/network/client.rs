use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum ClientMessage {
    RequestJoin,
    InputSync {
        last_acked_client_tick: u32,
        unacked_inputs: Vec<InputSample>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputSample {
    pub(crate) client_tick: u32,
    pub(crate) vertical_velocity: f32,
}

#[cfg(test)]
mod tests {
    use crate::pong::network::client::{ClientMessage, InputSample};

    #[test]
    fn encode_decode_equals() {
        let inputs: Vec<InputSample> = vec![];
        let cmd = ClientMessage::InputSync {
            last_acked_client_tick: 1,
            unacked_inputs: inputs.clone(),
        };
        let encoded = bincode::serialize(&cmd).unwrap();
        let decoded = bincode::deserialize(&encoded).unwrap();
        println!("{decoded:?}");
        assert!(
            matches!(
                decoded,
                ClientMessage::InputSync {
                    last_acked_client_tick: 1,
                    unacked_inputs: inputs,
                },
            ),
            "Decoded message does not equal original message"
        );
    }
}
