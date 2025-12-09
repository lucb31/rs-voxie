use glam::{Mat4, Vec3, Vec4Swizzles};
use hecs::World;
use log::debug;

use crate::{
    command_queue::{Command, CommandQueue},
    systems::physics::Transform,
};

pub struct Gun {
    // Remaining cooldown in s until we can fire again
    pub cooldown: f32,
    // Projectiles per s
    pub fire_rate: f32,
    pub triggered: bool,
}

pub fn system_gun_fire(world: &mut World, command_queue: &mut CommandQueue, dt: f32) {
    for (_entity, (transform_component, gun)) in world.query_mut::<(&Transform, &mut Gun)>() {
        gun.cooldown = 0.0f32.max(gun.cooldown - dt);
        if !gun.triggered {
            continue;
        }
        gun.triggered = false;
        if gun.cooldown > 0.0 {
            debug!("Reloading! {}ms cooldown remaining", gun.cooldown * 1e3);
            return;
        }
        let transform = transform_component.0;
        let forward = (-transform.z_axis.xyz()).normalize();
        let mut projectile_transform = transform;
        // Offset toward front of player
        projectile_transform.w_axis.x += forward.x * 2.0;
        projectile_transform.w_axis.y += forward.y * 2.0;
        projectile_transform.w_axis.z += forward.z * 2.0;
        // Scale by 0.4
        projectile_transform *= Mat4::from_scale(Vec3::splat(0.4));
        let velocity: Vec3 = forward * 40.0;

        debug!("Queuing up projectile");
        command_queue.enqueue(Command::SpawnProjectile {
            transform: projectile_transform,
            velocity,
        });
        gun.cooldown = 1.0 / gun.fire_rate;
    }
}
