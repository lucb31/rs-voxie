use std::collections::HashMap;

use hecs::{DynamicBundle, Entity, Query, World};
use log::{debug, trace};

use crate::systems::physics::Transform;

pub type NetEntityId = u32;

/// Simple wrapper around hecs::World to keep track of net entity id mapping
/// Supposed to be used by both client & server
pub struct NetworkWorld {
    world: World,
    next_net_entity: u32,
    network_to_local: HashMap<NetEntityId, Entity>,
    local_to_network: HashMap<Entity, NetEntityId>,
}

impl NetworkWorld {
    pub fn new() -> NetworkWorld {
        Self {
            world: World::new(),
            next_net_entity: 0,
            network_to_local: HashMap::new(),
            local_to_network: HashMap::new(),
        }
    }

    fn get_next_entity_id(&mut self) -> u32 {
        let next = self.next_net_entity;
        self.next_net_entity += 1;
        next
    }

    /// Spawn an entity on client or server
    /// Server: Will generate new net id
    /// Client: Will require net id to be provided
    pub fn spawn(
        &mut self,
        components: impl DynamicBundle,
        net_entity_id_option: Option<NetEntityId>,
    ) -> (NetEntityId, Entity) {
        let entity_id = self.world.spawn(components);
        let net_entity_id = match net_entity_id_option {
            // Client spawn
            Some(net_entity_id) => {
                debug_assert!(
                    self.get_entity_id(net_entity_id).is_none(),
                    "Duplicate use of net entity id {net_entity_id}"
                );
                net_entity_id
            }
            // Server spawn
            None => self.get_next_entity_id(),
        };
        self.network_to_local.insert(net_entity_id, entity_id);
        self.local_to_network.insert(entity_id, net_entity_id);
        (net_entity_id, entity_id)
    }

    pub fn get_world(&self) -> &World {
        &self.world
    }

    pub fn get_world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    pub fn query<Q: Query>(&self) -> hecs::QueryBorrow<'_, Q> {
        self.world.query::<Q>()
    }

    pub fn get_net_entity_id(&self, entity: &Entity) -> Option<&NetEntityId> {
        self.local_to_network.get(entity)
    }

    pub fn get_entity_id(&self, net_entity_id: NetEntityId) -> Option<&Entity> {
        self.network_to_local.get(&net_entity_id)
    }

    pub fn despawn_net_id(&mut self, net_entity_id: u32) -> Result<(), String> {
        let entity = self
            .network_to_local
            .remove(&net_entity_id)
            .ok_or("Could not find entity for net id {net_entity_id}")?;
        self.local_to_network.remove(&entity);
        self.world
            .despawn(entity)
            .map_err(|_| "Mapped entity id not found in ecs.".to_string())
    }

    pub fn despawn_all<T: hecs::Query>(&mut self) -> Result<(), String> {
        let to_despawn: Vec<hecs::Entity> = self.query::<T>().iter().map(|(e, _)| e).collect();
        debug!("Despawning entities: {to_despawn:?} ");
        for e in to_despawn {
            let net_id = self.local_to_network.remove(&e);
            self.network_to_local.remove(&net_id.unwrap());
            self.world
                .despawn(e)
                .map_err(|_| "Could not find entity".to_string())?;
        }
        Ok(())
    }
}
