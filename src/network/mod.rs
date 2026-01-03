mod client;
mod headless;
mod message;
mod meter;
mod server;
mod snapshot;
mod time_sync;
mod world;

pub enum Authority {
    Client(ClientId),
    Server,
}
pub struct NetworkReplicated {
    pub authority: Authority,
}

pub use client::NetworkClient;
pub use headless::HeadlessSimulation;
pub use server::ClientId;
pub use server::NetworkServer;
pub use server::ServerDownstreamPayload;
pub use server::ServerUpstreamPayload;
pub use snapshot::EntitySnapshot;
pub use snapshot::SnapshotManager;
pub use world::NetEntityId;
pub use world::NetworkWorld;
