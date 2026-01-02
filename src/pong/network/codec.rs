use std::fmt::Display;

use super::server::ServerMessage;

pub trait NetworkCodec {
    type Error: Display + std::fmt::Debug;

    fn encode(cmd: &ServerMessage) -> Result<Vec<u8>, Self::Error>;
    fn decode(input: &[u8]) -> Result<ServerMessage, Self::Error>;
}

pub struct JsonCodec;

impl NetworkCodec for JsonCodec {
    type Error = serde_json::Error;

    fn encode(cmd: &ServerMessage) -> Result<Vec<u8>, Self::Error> {
        todo!(
            "JSON encoding no longer fully supported. Client-side message have bincode hard-coded"
        );
        Ok(serde_json::to_string(cmd)?.into())
    }

    fn decode(input: &[u8]) -> Result<ServerMessage, Self::Error> {
        serde_json::from_str(&String::from_utf8_lossy(input))
    }
}

pub struct BincodeCodec;
impl NetworkCodec for BincodeCodec {
    type Error = Box<bincode::ErrorKind>;

    fn encode(cmd: &ServerMessage) -> Result<Vec<u8>, Self::Error> {
        bincode::serialize(cmd)
    }

    fn decode(input: &[u8]) -> Result<ServerMessage, Self::Error> {
        bincode::deserialize(input)
    }
}

#[cfg(test)]
mod tests {

    use crate::pong::network::{NetworkCodec, ServerMessage, codec::BincodeCodec};

    #[test]
    fn encode_decode_equals() {
        let cmd = ServerMessage::SpawnPlayer {
            player_net_entity: 5,
            player_slot: 0,
        };
        let encoded = BincodeCodec::encode(&cmd).unwrap();
        let decoded = BincodeCodec::decode(&encoded).unwrap();
        println!("{decoded:?}");
        assert!(
            matches!(
                decoded,
                ServerMessage::SpawnPlayer {
                    player_net_entity: 5,
                    player_slot: 0,
                }
            ),
            "Decoded message does not equal original message"
        );
    }
}
