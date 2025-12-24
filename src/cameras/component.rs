use glam::Mat4;
use hecs::{Entity, World};

use crate::{
    application::{RESOLUTION_HEIGHT, RESOLUTION_WIDTH},
    systems::physics::Transform,
};

pub struct CameraComponent {
    pub projection: Mat4,
}

pub fn spawn_camera(world: &mut World, transform: Mat4) -> Entity {
    world.spawn((
        Transform(transform),
        CameraComponent {
            projection: Mat4::perspective_rh_gl(
                60f32.to_radians(),
                RESOLUTION_WIDTH as f32 / RESOLUTION_HEIGHT as f32,
                0.1,
                1000.0,
            ),
        },
    ))
}
