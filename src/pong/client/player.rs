use glam::{Mat4, Quat, Vec3};
use hecs::{Entity, World};
use log::error;
use winit::keyboard::KeyCode;

use crate::{
    cameras::component::CameraComponent,
    input::InputState,
    network::{Authority, ClientId, NetEntityId, NetworkReplicated, NetworkWorld},
    pong::{
        common::{paddle::spawn_paddle, player::apply_input_buffer_sample},
        network::{
            client::{ClientMessage, InputSample},
            input::{ACK_BUFFER_SIZE, ClientInputBuffer},
        },
    },
    renderer::ecs_renderer::RenderColor,
    systems::physics::Transform,
};

pub(super) struct PongPlayer;

pub(super) fn adjust_player_camera(world: &mut World, player_slot: usize) {
    let camera_configs: [Mat4; 2] = [
        Mat4::from_translation(Vec3::X * 3.5),
        Mat4::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::from_rotation_y(180f32.to_radians()),
            Vec3::X * -3.5,
        ),
    ];
    let config = match camera_configs.get(player_slot) {
        Some(v) => v,
        None => {
            error!("Unable to adjust player camera: no config found");
            return;
        }
    };
    let mut query = world.query::<&mut Transform>().with::<&CameraComponent>();
    let cam = match query.iter().next() {
        Some(v) => v.1,
        None => {
            error!("Not camera component found");
            return;
        }
    };
    cam.0 = *config;
}

pub(super) fn spawn_player_client(
    world: &mut NetworkWorld,
    player_slot: usize,
    net_entity_id: Option<NetEntityId>,
    client_id: ClientId,
) -> (NetEntityId, Entity) {
    let (net_id, paddle) = spawn_paddle(world, player_slot, net_entity_id);
    world
        .get_world_mut()
        .insert(
            paddle,
            (
                RenderColor(Vec3::Y),
                PongPlayer,
                NetworkReplicated {
                    authority: Authority::Client(client_id),
                },
            ),
        )
        .expect("Could not add player. Missing paddle entity");
    (net_id, paddle)
}

/// Parse keyboard inputs to set paddle input velocity
pub(super) fn apply_player_input(world: &mut World, input: &ClientInputBuffer) {
    let entity = match world.query::<&PongPlayer>().iter().next() {
        Some(v) => v.0,
        None => {
            error!("Could not apply player input. No player entity found");
            return;
        }
    };
    let sample = match input.last() {
        Some(v) => v,
        None => {
            error!(
                "Could not find last sample. Input buffer probably empty. Forgot to sample first?"
            );
            return;
        }
    };
    apply_input_buffer_sample(world, sample, entity);
}

pub(super) fn sample_input(buf: &mut ClientInputBuffer, input: &InputState, client_tick: u32) {
    let mut vertical_velocity = 0.0;
    if input.is_key_pressed(&KeyCode::KeyW) {
        vertical_velocity += 1.0;
    }
    if input.is_key_pressed(&KeyCode::KeyS) {
        vertical_velocity -= 1.0;
    }
    let sample = InputSample {
        client_tick,
        vertical_velocity,
    };
    buf.input_buffer.push(sample);
}

pub(super) fn assemble_input_sync_cmd(buf: &ClientInputBuffer) -> ClientMessage {
    debug_assert!(
        buf.input_buffer.len() < ACK_BUFFER_SIZE,
        "Input buffer overflow"
    );
    ClientMessage::InputSync {
        last_acked_client_tick: buf.last_acked_client_tick,
        unacked_inputs: buf.input_buffer.clone(),
    }
}
