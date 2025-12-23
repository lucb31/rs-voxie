use std::{
    collections::HashMap,
    sync::mpsc::{Receiver, Sender},
};

use hecs::{DynamicBundle, Entity, Query, World};
use log::{error, trace, warn};

use crate::{log_err, pong::spawn_ball, systems::physics::Transform};

use super::NetworkCommand;

pub type NetEntityId = u32;

/// Ideally will be used by both client & server
pub struct NetworkWorld {
    world: World,
    next_net_entity: u32,
    network_to_local: HashMap<NetEntityId, Entity>,
    local_to_network: HashMap<Entity, NetEntityId>,

    // Used by clients to receive entity updates
    receiving_channel: Option<Receiver<NetworkCommand>>,
    // Used by server to broadcast entity updates
    broadcast_channel: Option<Sender<NetworkCommand>>,
}

impl NetworkWorld {
    pub fn new() -> NetworkWorld {
        Self {
            world: World::new(),
            next_net_entity: 0,
            network_to_local: HashMap::new(),
            local_to_network: HashMap::new(),
            receiving_channel: None,
            broadcast_channel: None,
        }
    }

    pub fn set_receiver(&mut self, rx: Receiver<NetworkCommand>) {
        self.receiving_channel = Some(rx);
    }
    pub fn set_broadcast(&mut self, tx: Sender<NetworkCommand>) {
        self.broadcast_channel = Some(tx);
    }

    /// Spawn an entity on the server and broadcast it to all clients
    pub fn spawn(
        &mut self,
        components: impl DynamicBundle,
        net_entity_id_option: Option<NetEntityId>,
    ) -> (NetEntityId, Entity) {
        let entity_id = self.world.spawn(components);
        let net_entity_id = match net_entity_id_option {
            Some(net_entity_id) => {
                debug_assert!(
                    self.receiving_channel.is_some(),
                    "Client spawn without recv channel"
                );
                net_entity_id
            }
            None => {
                // Server spawn
                debug_assert!(
                    self.broadcast_channel.is_some(),
                    "Client authority world tried to spawn net entity. Currently not supported or forgot to setup broadcast_channel"
                );
                let net_entity_id = self.next_net_entity;
                self.next_net_entity += 1;

                let tx = self.broadcast_channel.as_ref().unwrap();
                log_err!(
                    tx.send(NetworkCommand::ServerSpawn { net_entity_id }),
                    "Failed to broadcast entity spawn: {err}"
                );
                net_entity_id
            }
        };
        self.network_to_local.insert(net_entity_id, entity_id);
        self.local_to_network.insert(entity_id, net_entity_id);
        (net_entity_id, entity_id)
    }

    // TODO: Dangerous. Double check & refactor all occurences
    pub fn get_world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn query<Q: Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    pub fn client_sync(&mut self) {
        if self.receiving_channel.is_none() {
            return;
        }
        while let Ok(cmd) = self.receiving_channel.as_ref().unwrap().try_recv() {
            if let Err(err) = match cmd {
                NetworkCommand::ServerUpdateTransform {
                    net_entity_id,
                    transform,
                } => self.update_network_transform(net_entity_id, transform),
                NetworkCommand::ServerSpawn { net_entity_id } => self.spawn_ball(net_entity_id),
                NetworkCommand::ServerDespawnBall { net_entity_id } => todo!(),
                _ => Err("Unable to process network command: {err}".to_string()),
            } {
                error!("Unable to process network command: {err}");
            }
        }
    }

    fn spawn_ball(&mut self, net_entity_id: NetEntityId) -> Result<(), String> {
        // TODO: Hard-coded ball entity
        warn!("Hard coded ball spawn: {net_entity_id}");
        spawn_ball(self, Some(net_entity_id));
        Ok(())
    }

    fn update_network_transform(
        &mut self,
        net_entity_id: NetEntityId,
        updated_transform: Transform,
    ) -> Result<(), String> {
        trace!(
            "Processing UpdateTransform: Entity {net_entity_id}, Transform {}",
            updated_transform.0
        );
        let entity = self
            .network_to_local
            .get(&net_entity_id)
            .ok_or(format!("Unknown network entity id {net_entity_id}"))?;
        let mut transform = self
            .world
            .get::<&mut Transform>(*entity)
            .or(Err("Could not find transform for associated entity"))?;
        transform.0 = updated_transform.0;
        Ok(())
    }
}
