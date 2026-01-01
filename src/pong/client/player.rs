use glam::Vec3;
use hecs::{Entity, World};
use winit::keyboard::KeyCode;

use crate::{
    input::InputState,
    log_err,
    network::{NetEntityId, NetworkWorld},
    pong::{ClientProtocol, network::client::ClientMessage},
    renderer::ecs_renderer::RenderColor,
};

use super::paddle::{PaddleControl, PaddleSpeed, spawn_paddle};

pub struct PongPlayer;

pub fn spawn_player(
    world: &mut NetworkWorld,
    position: Vec3,
    net_entity_id: Option<NetEntityId>,
) -> (NetEntityId, Entity) {
    let (net_id, paddle) = spawn_paddle(world, position, net_entity_id);
    world
        .get_world_mut()
        .insert(paddle, (PongPlayer, RenderColor(Vec3::Y)))
        .expect("Could not add player. Missing paddle entity");
    (net_id, paddle)
}

/// Parse keyboard inputs to set paddle input velocity
pub fn sample_player_input(world: &mut World, input: &InputState) {
    for (_entity, (speed, control)) in world
        .query::<(&PaddleSpeed, &mut PaddleControl)>()
        .with::<&PongPlayer>()
        .iter()
    {
        // Parse inputs
        let mut input_velocity = Vec3::ZERO;
        if input.is_key_pressed(&KeyCode::KeyW) {
            input_velocity += Vec3::Y;
        }
        if input.is_key_pressed(&KeyCode::KeyS) {
            input_velocity -= Vec3::Y;
        }
        // Directly to max speed.
        // Improvement: Smoothing / acceleration
        control.input_velocity = input_velocity * speed.speed;
    }
}

pub fn sync_player_input(world: &NetworkWorld, protocol: &ClientProtocol) {
    let player = world
        .query::<&PaddleControl>()
        .with::<&PongPlayer>()
        .iter()
        .next()
        .map(|(entity, paddle)| (entity, paddle.input_velocity));
    if let Some((entity, input_velocity)) = player {
        let net_entity = world.get_net_entity_id(&entity).unwrap();
        log_err!(
            protocol.send_cmd(ClientMessage::UpdatePlayerInputVelocity {
                net_entity_id: *net_entity,
                input_velocity,
            }),
            "Unable to send player update: {err}"
        );
    }
}
