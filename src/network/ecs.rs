use std::{collections::HashMap, sync::mpsc::Receiver};

use glam::{Mat4, Quat, Vec3};
use hecs::{DynamicBundle, Entity, World};
use log::{error, trace, warn};

use crate::{
    collision::ColliderBody,
    pong::{MIN_SPEED, PongBall},
    renderer::{RenderMeshHandle, ecs_renderer::MESH_PROJECTILE_2D},
    systems::physics::Transform,
};

use super::NetworkCommand;

pub type NetEntityId = u32;

/// Client synchronization helper
pub struct EcsSynchronizer {
    network_cmd_queue: Receiver<NetworkCommand>,
    network_to_local: HashMap<NetEntityId, Entity>,
    local_to_network: HashMap<Entity, NetEntityId>,
}

impl EcsSynchronizer {
    pub fn new(rx: Receiver<NetworkCommand>) -> EcsSynchronizer {
        Self {
            network_cmd_queue: rx,
            network_to_local: HashMap::new(),
            local_to_network: HashMap::new(),
        }
    }

    pub fn sync(&mut self, world: &mut World) {
        while let Ok(cmd) = self.network_cmd_queue.try_recv() {
            if let Err(err) = match cmd {
                NetworkCommand::ServerUpdateTransform {
                    net_entity_id,
                    transform,
                } => self.update_network_transform(net_entity_id, transform, world),
                NetworkCommand::ServerSpawnBall { net_entity_id } => {
                    self.spawn_network_entity(net_entity_id, world)
                }
                NetworkCommand::ServerDespawnBall { net_entity_id } => todo!(),
                _ => Err("Unable to process network command: {err}".to_string()),
            } {
                error!("Unable to process network command: {err}");
            }
        }
    }

    fn spawn_network_entity(
        &mut self,
        net_entity_id: NetEntityId,
        world: &mut World,
    ) -> Result<(), String> {
        // TODO: Hard-coded ball entity
        warn!("Hard coded ball spawn");
        let ball_entity = spawn_ball_client(world);
        self.local_to_network.insert(ball_entity, net_entity_id);
        self.network_to_local.insert(net_entity_id, ball_entity);
        Ok(())
    }

    fn update_network_transform(
        &mut self,
        net_entity_id: NetEntityId,
        updated_transform: Transform,
        world: &mut World,
    ) -> Result<(), String> {
        trace!(
            "Processing UpdateTransform: Entity {net_entity_id}, Transform {}",
            updated_transform.0
        );
        let entity = self
            .network_to_local
            .get(&net_entity_id)
            .ok_or("Unknown network entity id")?;
        let mut transform = world
            .get::<&mut Transform>(*entity)
            .or(Err("Could not find transform for associated entity"))?;
        transform.0 = updated_transform.0;
        Ok(())
    }
}

fn spawn_ball_client(world: &mut World) -> Entity {
    let scale = Vec3::splat(0.25);
    let speed = MIN_SPEED;
    world.spawn((
        PongBall { speed, bounces: 0 },
        Transform(Mat4::from_scale_rotation_translation(
            scale,
            Quat::IDENTITY,
            Vec3::ZERO,
        )),
        RenderMeshHandle(MESH_PROJECTILE_2D),
        ColliderBody::SphereCollider { radius: 0.125 },
    ))
}

pub struct ServerWorld {
    pub world: World,
    next_net_entity: u32,
    net_to_local: HashMap<NetEntityId, Entity>,
}

impl ServerWorld {
    pub fn new() -> ServerWorld {
        Self {
            world: World::new(),
            next_net_entity: 0,
            net_to_local: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, components: impl DynamicBundle) -> (NetEntityId, Entity) {
        let entity_id = self.world.spawn(components);
        let net_entity_id = self.next_net_entity;
        self.net_to_local.insert(net_entity_id, entity_id);
        self.next_net_entity += 1;
        (net_entity_id, entity_id)
    }
}
