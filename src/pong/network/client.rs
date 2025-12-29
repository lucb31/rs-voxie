use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cmd", content = "data")]
pub enum ClientMessage {
    StartRound,
    Ping { timestamp: u128 },
}
