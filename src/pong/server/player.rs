use log::{debug, error, warn};

use crate::{network::NetworkWorld, pong::common::player::apply_input_buffer_sample};

use super::lobby::Lobby;

pub(super) fn apply_player_inputs(world: &mut NetworkWorld, lobby: &mut Lobby) {
    for player in lobby.iter_players_mut() {
        let sample = match player.input_buffer.get_oldest() {
            Some(v) => v,
            None => {
                warn!("No input buffered for client: {}", player.client_id);
                continue;
            }
        };
        let net_entity_id = match player.player_net_id {
            Some(v) => v,
            None => {
                error!("Unknown net entity id for player");
                continue;
            }
        };
        let player_entity = match world.get_entity_id(net_entity_id).copied() {
            Some(v) => v,
            None => {
                error!("Unknown net entity {net_entity_id}");
                continue;
            }
        };
        apply_input_buffer_sample(world.get_world_mut(), sample, player_entity);
        debug!(
            "Applying sample at client tick {} for player {}. {} samples remaining",
            sample.client_tick,
            player.client_id,
            player.input_buffer.get_buffer_size()
        );
        player
            .input_buffer
            .update_acked_client_tick(sample.client_tick);
    }
}
