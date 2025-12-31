use glam::Vec4Swizzles;
use log::{debug, error, info};

use crate::{
    log_err,
    network::{ClientId, EntitySnapshot, NetworkReplicated, NetworkWorld},
    pong::{
        JsonCodec,
        client::{
            ball::{PongBall, spawn_ball},
            paddle::PaddleControl,
            player::spawn_player,
        },
        network::{ServerMessage, client::ClientMessage},
    },
    systems::physics::Transform,
};

use super::{lobby::Lobby, protocol::ServerProtocol};

pub fn server_process_client_message(
    world: &mut NetworkWorld,
    msg: (ClientMessage, ClientId),
    protocol: &ServerProtocol<JsonCodec>,
    game_over: &mut bool,
    lobby: &mut Lobby,
    frame: u32,
) {
    let (cmd, client) = msg;
    debug!("Server received cmd {cmd:?} from {client}");
    let result: Result<(), String> = (|| match cmd {
        ClientMessage::RequestJoin => {
            if world.query::<&PongBall>().iter().next().is_some() {
                return Err("Game is still in progress. Cannot spawn new ball".to_string());
            } else if !*game_over {
                return Err("Join requested while game in progress".to_string());
            }

            // Spawn player
            let player_position = lobby.join(client)?;
            let (player_net_entity, player_entity_id) = spawn_player(world, player_position, None);
            protocol.send_to(
                ServerMessage::SpawnPlayer {
                    player_net_entity,
                    position: player_position,
                },
                client,
            )?;

            // Spawn paddle of new player in other clients
            for other_player in lobby.others(client) {
                protocol.send_to(
                    ServerMessage::SpawnPaddle {
                        net_entity_id: player_net_entity,
                        position: player_position,
                    },
                    other_player,
                )?;
            }
            // Spawn paddles of other client for new player
            for (paddle_entity, transform) in
                world.query::<&Transform>().with::<&PaddleControl>().iter()
            {
                if paddle_entity == player_entity_id {
                    // Skip our own paddle
                    continue;
                }
                let net_entity_id = world
                    .get_net_entity_id(&paddle_entity)
                    .ok_or("Invalid net entity mapping for paddle".to_string())?;
                protocol.send_to(
                    ServerMessage::SpawnPaddle {
                        net_entity_id: *net_entity_id,
                        position: transform.0.w_axis.xyz(),
                    },
                    client,
                )?;
            }

            // Start game if final player joined
            if lobby.is_ready() {
                info!("Player {client} joined. Lobby is ready. Starting round");
                *game_over = false;
                let (ball_net_id, _entity) = spawn_ball(world, None);
                protocol.broadcast(ServerMessage::StartRound {
                    ball_net_entity: ball_net_id,
                    frame,
                })
            } else {
                info!("Player {client} joined. Waiting for more players to join...");
                Ok(())
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
        ClientMessage::Ping { timestamp } => {
            protocol.send_to(ServerMessage::Pong { timestamp }, client)
        }
    })();
    if let Err(err) = result {
        error!("Server failed to process cmd {cmd:?}: {err}");
    }
}

pub fn server_broadcast_transform_state(
    world: &NetworkWorld,
    protocol: &ServerProtocol<JsonCodec>,
    frame: u32,
) {
    let mut snapshots: Vec<EntitySnapshot> = Vec::new();
    for (entity, transform) in world
        .get_world()
        .query::<&Transform>()
        .with::<&NetworkReplicated>()
        .iter()
    {
        match world.get_net_entity_id(&entity) {
            Some(net_entity_id) => {
                snapshots.push(EntitySnapshot {
                    net_entity_id: *net_entity_id,
                    transform: transform.clone(),
                });
            }
            None => {
                error!(
                    "Failed to broadcast transform for entity {entity:?}: No net entity id found"
                );
            }
        }
    }
    // Sort by net entity id so we can binary search when processing snapshot
    snapshots.sort_unstable_by(|a, b| a.net_entity_id.partial_cmp(&b.net_entity_id).unwrap());
    log_err!(
        protocol.broadcast(ServerMessage::SendSnapshot {
            frame,
            data: snapshots
        }),
        "Failure broadcasting command: {err}"
    );
}
