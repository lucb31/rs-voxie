use glam::Vec3;
use hecs::{Entity, World};

use crate::{
    log_err,
    network::{Authority, ClientId, NetEntityId, NetworkReplicated, NetworkWorld},
    pong::{common::paddle::PaddleControl, network::client::InputSample},
};

use super::paddle::spawn_paddle;

pub(crate) fn apply_input_buffer_sample(
    world: &mut World,
    sample: &InputSample,
    player_entity: Entity,
) {
    // Directly to max speed.
    // Improvement: Smoothing / acceleration
    let input_velocity = Vec3::Y * sample.vertical_velocity;
    log_err!(
        world.exchange_one::<PaddleControl, PaddleControl>(
            player_entity,
            PaddleControl { input_velocity },
        ),
        "Failed to update paddle input velocity: {err}"
    );
}

pub(crate) fn spawn_player(
    world: &mut NetworkWorld,
    player_slot: usize,
    net_entity_id: Option<NetEntityId>,
    client_id: ClientId,
) -> (NetEntityId, Entity) {
    let (net_id, paddle) = spawn_paddle(world, player_slot, net_entity_id);
    world
        .get_world_mut()
        .insert(
            paddle,
            (NetworkReplicated {
                authority: Authority::Client(client_id),
            },),
        )
        .expect("Could not add player. Missing paddle entity");
    (net_id, paddle)
}
