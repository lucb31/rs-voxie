use glam::Vec3;
use hecs::{Entity, World};
use log::error;
use winit::keyboard::KeyCode;

use crate::{
    input::InputState,
    network::{Authority, ClientId, NetEntityId, NetworkReplicated, NetworkWorld},
    pong::{common::player::apply_input_buffer_sample, network::input::ClientInputBuffer},
    renderer::ecs_renderer::RenderColor,
};

use crate::pong::common::paddle::{PaddleControl, PaddleSpeed, spawn_paddle};

pub struct PongPlayer;

pub fn spawn_player(
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
            (
                PongPlayer,
                RenderColor(Vec3::Y),
                NetworkReplicated {
                    authority: Authority::Client(client_id),
                },
            ),
        )
        .expect("Could not add player. Missing paddle entity");
    (net_id, paddle)
}

/// Parse keyboard inputs to set paddle input velocity
pub fn apply_player_input(world: &mut World, input: &ClientInputBuffer) {
    let entity = match world.query::<&PongPlayer>().iter().next() {
        Some(v) => v.0,
        None => {
            error!("Could not apply player input. No player entity found");
            return;
        }
    };
    let sample = match input.last() {
        Some(v) => v,
        None => {
            error!(
                "Could not find last sample. Input buffer probably empty. Forgot to sample first?"
            );
            return;
        }
    };
    apply_input_buffer_sample(world, sample, entity);
}
