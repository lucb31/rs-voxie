use glam::Vec3;
use log::{debug, error, info, trace};

use crate::{
    log_err,
    network::{ClientId, EntitySnapshot, NetworkReplicated, NetworkWorld},
    pong::{
        BincodeCodec,
        client::player::spawn_player,
        common::{
            ball::{BALL_MIN_SPEED, PongBall, spawn_ball},
            paddle::{PaddleControl, PaddleId},
        },
        network::{ServerMessage, client::ClientMessage},
    },
    systems::physics::{Transform, Velocity},
};

use super::{lobby::Lobby, protocol::ServerProtocol, scene::ServerGameState};

pub(super) fn server_process_client_message(
    world: &mut NetworkWorld,
    msg: (ClientMessage, ClientId),
    protocol: &ServerProtocol<BincodeCodec>,
    game_state: &mut ServerGameState,
    lobby: &mut Lobby,
    frame: u32,
) {
    let (cmd, client) = msg;
    trace!("Server received cmd {cmd:?} from {client}");
    let result: Result<(), String> = (|| match &cmd {
        ClientMessage::RequestJoin => {
            if world.query::<&PongBall>().iter().next().is_some() {
                return Err("Game is still in progress. Cannot spawn new ball".to_string());
            } else if !matches!(game_state, ServerGameState::WaitingForPlayers) {
                return Err(
                    "Join requested, but server does not accept new players right now".to_string(),
                );
            }

            // Spawn player
            let player_slot = lobby.join(client)?;
            let (player_net_entity, player_entity_id) =
                spawn_player(world, player_slot, None, client);
            let player_info = lobby
                .get_player_info_mut(client)
                .ok_or("Failed to associate player info".to_string())?;
            player_info.player_net_id = Some(player_net_entity);
            protocol.send_to(
                ServerMessage::SpawnPlayer {
                    player_net_entity,
                    player_slot,
                },
                client,
            )?;

            // Spawn paddle of new player in other clients
            for other_player in lobby.others(client) {
                protocol.send_to(
                    ServerMessage::SpawnPaddle {
                        net_entity_id: player_net_entity,
                        player_slot,
                    },
                    other_player,
                )?;
            }
            // Spawn paddles of other client for new player
            for (paddle_entity, paddle_id) in world.query::<&PaddleId>().iter() {
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
                        player_slot: paddle_id.slot,
                    },
                    client,
                )?;
            }

            // Start game if final player joined
            if lobby.is_ready() {
                info!("Player {client} joined. Lobby is ready. Starting round");
                *game_state = ServerGameState::Running;
                let (ball_net_entity, entity) = spawn_ball(world, None);
                let direction = Vec3::new(1.0, 0.5, 0.0).normalize();
                log_err!(
                    world
                        .get_world_mut()
                        .insert(entity, (Velocity(direction * BALL_MIN_SPEED),)),
                    "Could not add ball speed {err}"
                );
                protocol.broadcast(ServerMessage::StartRound {
                    ball_net_entity,
                    server_tick: frame,
                })
            } else {
                info!("Player {client} joined. Waiting for more players to join...");
                Ok(())
            }
        }
        ClientMessage::InputSync {
            last_acked_client_tick,
            unacked_inputs,
        } => {
            // Store client provided inputs in server-side copy
            match lobby.get_player_info_mut(client) {
                Some(player_info) => {
                    player_info
                        .input_buffer
                        .set_buffer(unacked_inputs.to_owned());
                }
                None => {
                    error!("Cannot process player input. Could not find player info");
                }
            }
            Ok(())
        }
        ClientMessage::UpdatePlayerInputVelocity {
            net_entity_id,
            input_velocity,
        } => {
            debug!("processing input at frame {frame}");
            let entity = world
                .get_entity_id(*net_entity_id)
                .ok_or("Unknown net entity {net_entity_id}")
                .copied()?;
            world
                .get_world_mut()
                .exchange_one::<PaddleControl, PaddleControl>(
                    entity,
                    PaddleControl {
                        input_velocity: *input_velocity,
                    },
                )
                .map_err(|err| "Failed to update paddle input velocity: {err}".to_string())?;
            Ok(())
        }
    })();
    if let Err(err) = result {
        error!("Server failed to process cmd {cmd:?}: {err}");
    }
}

pub(super) fn server_send_snapshots(
    world: &NetworkWorld,
    protocol: &ServerProtocol<BincodeCodec>,
    lobby: &Lobby,
    server_tick: u32,
) {
    // Create global snapshot of replicated entities to be used by all clients
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

    // Send to player including last acked tick
    for player in lobby.iter_players() {
        log_err!(
            protocol.send_to(
                ServerMessage::SendSnapshot {
                    server_tick,
                    data: snapshots.clone(),
                    last_acked_client_tick: player.input_buffer.get_last_acked()
                },
                player.client_id
            ),
            "Failure broadcasting command: {err}"
        );
    }
}
