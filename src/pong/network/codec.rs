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
        Ok(serde_json::to_string(cmd)?.into())
    }

    fn decode(input: &[u8]) -> Result<ServerMessage, Self::Error> {
        serde_json::from_str(&String::from_utf8_lossy(input))
    }
}
