mod client;
pub(super) mod network;
pub mod server;

pub use client::protocol::ClientProtocol;
pub use client::scene::PongScene;
pub use network::BincodeCodec;
pub use network::JsonCodec;
pub use server::protocol::ServerProtocol;
pub use server::scene::PongServerScene;
