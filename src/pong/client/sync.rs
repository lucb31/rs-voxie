use log::{debug, error, info};

use crate::{
    log_err,
    network::NetworkWorld,
    pong::{
        client::{
            ai::spawn_ai,
            ball::{PongBall, spawn_ball},
            paddle::PongPaddle,
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
        ServerMessage::ServerUpdateTransform {
            net_entity_id,
            transform,
        } => world.update_transform_by_net_id(net_entity_id, transform),
        ServerMessage::ServerStartRound {
            ball_net_entity,
            ai_net_entity,
        } => {
            *game_over = false;
            spawn_ball(world, Some(ball_net_entity));
            spawn_ai(world, Some(ai_net_entity));
            Ok(())
        }
        ServerMessage::ServerDespawnEntity { net_entity_id } => world.despawn_net_id(net_entity_id),
        ServerMessage::ServerEndRound { winner } => {
            info!("According to the server the winner is {winner}");
            *game_over = true;
            log_err!(
                world.despawn_all::<&PongBall>(),
                "Could not despawn balls {err}"
            );
            log_err!(
                world.despawn_all::<&PongPaddle>(),
                "Could not despawn paddles {err}"
            );
            Ok(())
        }
        _ => Err("Unable to process network command: {cmd:?}".to_string()),
    } {
        error!("Unable to process network command: {err}");
    }
}
