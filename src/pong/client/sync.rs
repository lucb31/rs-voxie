use log::{debug, error, info};

use crate::{
    log_err,
    network::NetworkWorld,
    pong::{
        client::{
            ball::{PongBall, spawn_ball},
            paddle::{PaddleControl, spawn_paddle},
            player::spawn_player,
        },
        network::ServerMessage,
    },
};

pub fn client_handle_network_cmd(
    world: &mut NetworkWorld,
    cmd: ServerMessage,
    game_over: &mut bool,
) {
    debug!("Client received cmd {cmd:?}");
    if let Err(err) = match cmd {
        ServerMessage::UpdateTransform {
            net_entity_id,
            transform,
        } => world.update_transform_by_net_id(net_entity_id, transform),
        ServerMessage::StartRound { ball_net_entity } => {
            *game_over = false;
            spawn_ball(world, Some(ball_net_entity));
            Ok(())
        }
        ServerMessage::SpawnPlayer {
            player_net_entity,
            position,
        } => {
            spawn_player(world, position, Some(player_net_entity));
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
        ServerMessage::EndRound { winner } => {
            info!("According to the server the winner is {winner}");
            *game_over = true;
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
        ServerMessage::Pong { timestamp } => todo!("Ping implementation missing"),
    } {
        error!("Unable to process network command: {err}");
    }
}
