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
            *game_state = GameState::Running;
            spawn_ball(world, Some(ball_net_entity));
            Ok(())
        }
        ServerMessage::EndRound { winner } => {
            info!("According to the server the winner is {winner}");
            *game_state = GameState::Initial;
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
        ServerMessage::Pong { timestamp } => {
            Err("Ping should be handled in protocol layer".to_string())
        }
    } {
        error!("Unable to process network command: {err}");
    }
}
