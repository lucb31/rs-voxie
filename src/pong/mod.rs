#[cfg(feature = "gui")]
pub mod client;
pub(super) mod common;
pub(super) mod network;
pub mod server;

#[cfg(feature = "gui")]
pub use client::protocol::ClientProtocol;
pub use network::BincodeCodec;
pub use server::protocol::ServerProtocol;
