use glam::{Mat4, Quat, Vec3};
use log::{error, info, trace};

use crate::{
    cameras::component::CameraComponent,
    network::{NetworkWorld, SnapshotManager},
    pong::{
        ClientProtocol,
        client::{
            player::{adjust_player_camera, spawn_player},
            scene::GameOverTransition,
        },
        common::{ball::spawn_ball, paddle::spawn_paddle},
        network::{ServerMessage, input::ClientInputBuffer},
    },
    systems::physics::Transform,
};

use super::scene::GameState;

pub(super) fn client_handle_network_cmd(
    world: &mut NetworkWorld,
    cmd: ServerMessage,
    game_state: &mut GameState,
    snapshot_manager: &mut SnapshotManager,
    client: &ClientProtocol,
    input_buffer: &mut ClientInputBuffer,
) {
    trace!("Client received cmd {cmd:?}");
    if let Err(err) = match cmd {
        ServerMessage::SendSnapshot {
            server_tick,
            data,
            last_acked_client_tick,
        } => {
            // Store snapshot for interpolation buffering
            snapshot_manager.store_snapshot(server_tick, data, client.get_rtt_estimate());

            input_buffer.update_acked_client_tick(last_acked_client_tick);

            // TODO: Apply snapshot to authorative ecs
            // TODO: Separate rendering from simulation transform
            Ok(())
        }
        ServerMessage::StartRound {
            ball_net_entity,
            server_tick,
        } => {
            if let GameState::WaitingForOthers { player_slot } = game_state {
                *game_state = GameState::Running {
                    player_slot: *player_slot,
                    started_at_server_tick: server_tick,
                    end_signal: None,
                };
                *input_buffer = ClientInputBuffer::new();
                spawn_ball(world, Some(ball_net_entity));
                Ok(())
            } else {
                Err("Trying to start before waiting for others".to_string())
            }
        }
        ServerMessage::EndRound {
            server_tick,
            loosing_player_slot,
        } => {
            info!(
                "Game over: Player slot {loosing_player_slot} lost the game at tick {server_tick}"
            );
            if let GameState::Running { end_signal, .. } = game_state {
                *end_signal = Some(GameOverTransition::new(server_tick, loosing_player_slot));
                Ok(())
            } else {
                Err("Received end round signal while game was not running".to_string())
            }
        }
        ServerMessage::SpawnPlayer {
            player_net_entity,
            player_slot,
        } => {
            if let Some(client_id) = client.get_client_id() {
                spawn_player(world, player_slot, Some(player_net_entity), client_id);
                adjust_player_camera(world.get_world_mut(), player_slot);
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
