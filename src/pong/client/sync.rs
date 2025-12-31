use log::{debug, error, info};

use crate::{
    log_err,
    network::{NetworkWorld, SnapshotManager},
    pong::{
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
    snapshot_manager: &mut Option<SnapshotManager>,
) {
    debug!("Client received cmd {cmd:?}");
    if let Err(err) = match cmd {
        ServerMessage::SendSnapshot { frame, data } => match snapshot_manager {
            Some(manager) => {
                manager.store_snapshot(frame, data);
                Ok(())
            }
            None => Err("Received snapshot, but snapshot manager is not initialized".to_string()),
        },
        ServerMessage::StartRound {
            ball_net_entity,
            frame: server_frame,
        } => {
            *game_state = GameState::Running;
            *snapshot_manager = Some(SnapshotManager::new(server_frame));
            spawn_ball(world, Some(ball_net_entity));
            Ok(())
        }
        ServerMessage::EndRound { winner } => {
            info!("According to the server the winner is {winner}");
            *game_state = GameState::Initial;
            *snapshot_manager = None;
            log_err!(
                world.despawn_all::<&PongBall>(),
                "Could not despawn balls {err}"
            );
            log_err!(
                world.despawn_all::<&PaddleControl>(),
                "Could not despawn paddles {err}"
            );
            Ok(())
        }
        ServerMessage::SpawnPlayer {
            player_net_entity,
            position,
        } => {
            spawn_player(world, position, Some(player_net_entity));
            *game_state = GameState::WaitingForOthers;
            Ok(())
        }
        ServerMessage::SpawnPaddle {
            net_entity_id,
            position,
        } => {
            spawn_paddle(world, position, Some(net_entity_id));
            Ok(())
        }
        ServerMessage::DespawnEntity { net_entity_id } => world.despawn_net_id(net_entity_id),
        ServerMessage::Pong { timestamp } => todo!("Ping implementation missing"),
    } {
        error!("Unable to process network command: {err}");
    }
}
