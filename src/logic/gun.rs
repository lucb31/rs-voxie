use std::{cell::RefCell, rc::Rc};

use glam::{Mat4, Vec3, Vec4Swizzles};
use log::debug;

use crate::command_queue::{Command, CommandQueue};

pub struct Gun {
    command_queue: Rc<RefCell<CommandQueue>>,
    // Remaining cooldown in s until we can fire again
    cooldown: f32,
    // Projectiles per s
    fire_rate: f32,
}

impl Gun {
    pub fn new(command_queue: &Rc<RefCell<CommandQueue>>) -> Gun {
        Self {
            command_queue: Rc::clone(command_queue),
            cooldown: 0.0,
            fire_rate: 1.0,
        }
    }

    pub fn fire(&mut self, transform: &Mat4) {
        if self.cooldown > 0.0 {
            debug!(
                "Reloading! {}ms until we can shoot again",
                self.cooldown * 1e3
            );
            return;
        }
        let forward = (-transform.z_axis.xyz()).normalize();
        let mut projectile_transform = *transform;
        // Offset toward front of player
        projectile_transform.w_axis.x += forward.x * 2.0;
        projectile_transform.w_axis.y += forward.y * 2.0;
        projectile_transform.w_axis.z += forward.z * 2.0;
        // Scale by 0.4
        projectile_transform *= Mat4::from_scale(Vec3::splat(0.4));
        let velocity = forward * 20.0;
        self.command_queue
            .borrow_mut()
            .enqueue(Command::SpawnProjectile {
                transform: projectile_transform,
                velocity,
            });
        self.cooldown = 1.0 / self.fire_rate;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.cooldown > 0.0 {
            self.cooldown = (self.cooldown - dt).max(0.0);
        }
    }
}
