use log::{debug, error, info};

use crate::{
    log_err,
    network::{NetworkWorld, SnapshotManager},
    pong::{
        ClientProtocol,
        client::{
            ball::{PongBall, spawn_ball},
            paddle::{PaddleControl, spawn_paddle},
            player::spawn_player,
        },
        network::ServerMessage,
    },
};

use super::scene::GameState;

pub(super) fn client_handle_network_cmd(
    world: &mut NetworkWorld,
    cmd: ServerMessage,
    game_state: &mut GameState,
    snapshot_manager: &mut SnapshotManager,
    client: &ClientProtocol,
) {
    debug!("Client received cmd {cmd:?}");
    if let Err(err) = match cmd {
        ServerMessage::SendSnapshot { frame, data } => {
            // Store snapshot for interpolation buffering
            snapshot_manager.store_snapshot(frame, data, client.get_rtt_estimate());
            // TODO: Apply snapshot to authorative ecs
            Ok(())
        }
        ServerMessage::StartRound {
            ball_net_entity,
            frame,
        } => {
            if let GameState::WaitingForOthers { player_slot } = game_state {
                *game_state = GameState::Running {
                    player_slot: *player_slot,
                };
                spawn_ball(world, Some(ball_net_entity));
                Ok(())
            } else {
                Err("Trying to start before waiting for others".to_string())
            }
        }
        ServerMessage::EndRound {
            loosing_player_slot,
        } => {
            info!("Game over: Player slot {loosing_player_slot} lost the game");
            if let GameState::Running { player_slot } = game_state {
                *game_state = GameState::GameOver {
                    winner: loosing_player_slot != *player_slot,
                };
                log_err!(
                    world.despawn_all::<&PongBall>(),
                    "Could not despawn balls {err}"
                );
                log_err!(
                    world.despawn_all::<&PaddleControl>(),
                    "Could not despawn paddles {err}"
                );
                Ok(())
            } else {
                Err("Trying to end before running".to_string())
            }
        }
        ServerMessage::SpawnPlayer {
            player_net_entity,
            player_slot,
        } => {
            if let Some(client_id) = client.get_client_id() {
                spawn_player(world, player_slot, Some(player_net_entity), client_id);
                *game_state = GameState::WaitingForOthers { player_slot };
                Ok(())
            } else {
                Err("Cannot spawn player. Missing client id".to_string())
            }
        }
        ServerMessage::SpawnPaddle {
            net_entity_id,
            player_slot,
        } => {
            spawn_paddle(world, player_slot, Some(net_entity_id));
            Ok(())
        }
        ServerMessage::DespawnEntity { net_entity_id } => world.despawn_net_id(net_entity_id),
    } {
        error!("Unable to process network command: {err}");
    }
}
