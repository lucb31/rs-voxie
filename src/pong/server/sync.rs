use log::{debug, error, warn};

use crate::{
    log_err,
    network::{NetEntityId, NetworkReplicated, NetworkWorld},
    pong::{
        JsonCodec,
        client::{
            ai::spawn_ai,
            ball::{PongBall, spawn_ball},
            paddle::PaddleControl,
            player::spawn_player,
        },
        network::{ServerMessage, client::ClientMessage},
    },
    systems::physics::Transform,
};

use super::protocol::ServerProtocol;

pub fn server_process_client_message(
    world: &mut NetworkWorld,
    cmd: ClientMessage,
    protocol: &ServerProtocol<JsonCodec>,
    game_over: &mut bool,
) {
    debug!("Server received cmd {cmd:?}");
    let result: Result<(), String> = (|| match cmd {
        ClientMessage::StartRound => {
            if world.query::<&PongBall>().iter().next().is_some() {
                Err("Game is still in progress. Cannot spawn new ball".to_string())
            } else {
                *game_over = false;
                let (ball_net_id, _entity) = spawn_ball(world, None);
                let (ai_net_id, _) = spawn_ai(world, None);
                let (player_net_id, _) = spawn_player(world, None);
                protocol.broadcast(ServerMessage::StartRound {
                    ball_net_entity: ball_net_id,
                    ai_net_entity: ai_net_id,
                    player_net_entity: player_net_id,
                })
            }
        }
        ClientMessage::UpdatePlayerInputVelocity {
            net_entity_id,
            input_velocity,
        } => {
            let entity = world
                .get_entity_id(net_entity_id)
                .ok_or("Unknown net entity {net_entity_id}")
                .copied()?;
            world
                .get_world_mut()
                .exchange_one::<PaddleControl, PaddleControl>(
                    entity,
                    PaddleControl { input_velocity },
                )
                .map_err(|err| "Failed to update paddle input velocity: {err}".to_string())?;
            Ok(())
        }
        ClientMessage::Ping { timestamp } => todo!("Ping currently not implemented"),
    })();
    if let Err(err) = result {
        error!("Server failed to process cmd {cmd:?}: {err}");
    }
}

pub fn server_broadcast_transform_state(
    world: &NetworkWorld,
    protocol: &ServerProtocol<JsonCodec>,
) {
    for (entity, transform) in world
        .get_world()
        .query::<&Transform>()
        .with::<&NetworkReplicated>()
        .iter()
    {
        match world.get_net_entity_id(&entity) {
            Some(net_entity_id) => {
                let cmd: ServerMessage = ServerMessage::UpdateTransform {
                    net_entity_id: *net_entity_id,
                    transform: transform.clone(),
                };
                log_err!(
                    protocol.broadcast(cmd),
                    "Failure broadcasting command: {err}"
                );
            }
            None => {
                error!(
                    "Failed to broadcast transform for entity {entity:?}: No net entity id found"
                );
            }
        }
    }
}
