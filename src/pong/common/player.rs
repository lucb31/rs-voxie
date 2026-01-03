use glam::Vec3;
use hecs::{Entity, World};

use crate::{
    log_err,
    pong::{common::paddle::PaddleControl, network::client::InputSample},
};

pub(crate) fn apply_input_buffer_sample(
    world: &mut World,
    sample: &InputSample,
    player_entity: Entity,
) {
    // Directly to max speed.
    // Improvement: Smoothing / acceleration
    let input_velocity = Vec3::Y * sample.vertical_velocity;
    log_err!(
        world.exchange_one::<PaddleControl, PaddleControl>(
            player_entity,
            PaddleControl { input_velocity },
        ),
        "Failed to update paddle input velocity: {err}"
    );
}
