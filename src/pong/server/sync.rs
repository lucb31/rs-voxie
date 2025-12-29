use log::{debug, error, warn};

use crate::{
    log_err,
    network::{NetworkReplicated, NetworkWorld},
    pong::{
        JsonCodec,
        client::{
            ai::spawn_ai,
            ball::{PongBall, spawn_ball},
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
    match cmd {
        ClientMessage::StartRound => {
            if world.query::<&PongBall>().iter().next().is_some() {
                warn!("Game is still in progress. Cannot spawn new ball");
            } else {
                *game_over = false;
                let (ball_net_id, _entity) = spawn_ball(world, None);
                let (ai_net_id, _) = spawn_ai(world, None);
                protocol
                    .broadcast(ServerMessage::ServerStartRound {
                        ball_net_entity: ball_net_id,
                        ai_net_entity: ai_net_id,
                    })
                    .expect("Broadcast spawn failed");
            }
        }
        _ => {
            error!("Server does not know how to handle this command: {cmd:?}");
        }
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
                let cmd: ServerMessage = ServerMessage::ServerUpdateTransform {
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
