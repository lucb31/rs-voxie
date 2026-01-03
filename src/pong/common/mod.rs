use boundary::spawn_boundaries;
use glam::{Mat4, Vec3};

pub(crate) mod ball;
pub(super) mod boundary;
pub(crate) mod paddle;

use crate::{
    cameras::component::CameraComponent, network::NetworkWorld, systems::physics::Transform,
};

pub(crate) fn setup_static_entities(world: &mut NetworkWorld) {
    // Spawn camera directly into world -> No replication
    let scale_y = 3.5;
    let scale_x = scale_y * 16.0 / 9.0;
    let projection =
        Mat4::orthographic_rh_gl(-scale_x, scale_x, -scale_y, scale_y, -scale_y, scale_y);
    world.get_world_mut().spawn((
        Transform(Mat4::from_translation(Vec3::X * 3.5)),
        CameraComponent { projection },
    ));

    // Spawn boundaries directly into world -> No replication required
    let width = 5.0;
    let height = 5.0;
    spawn_boundaries(world.get_world_mut(), width, height);
}
