use glam::{Mat4, Quat, Vec3};
use hecs::{Entity, World};
use log::error;

use crate::{
    cameras::component::CameraComponent,
    network::{Authority, ClientId, NetEntityId, NetworkReplicated, NetworkWorld},
    pong::{common::player::apply_input_buffer_sample, network::input::ClientInputBuffer},
    renderer::ecs_renderer::RenderColor,
    systems::physics::Transform,
};

use crate::pong::common::paddle::spawn_paddle;

pub struct PongPlayer;

pub fn adjust_player_camera(world: &mut World, player_slot: usize) {
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

pub fn spawn_player(
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
                PongPlayer,
                RenderColor(Vec3::Y),
                NetworkReplicated {
                    authority: Authority::Client(client_id),
                },
            ),
        )
        .expect("Could not add player. Missing paddle entity");
    (net_id, paddle)
}

/// Parse keyboard inputs to set paddle input velocity
pub fn apply_player_input(world: &mut World, input: &ClientInputBuffer) {
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
