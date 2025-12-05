use std::{cell::RefCell, collections::HashMap, rc::Rc};

use glam::{Mat4, Vec3};
use glow::Context;
use log::debug;
use uuid::Uuid;

use crate::{cameras::camera::Camera, command_queue::CommandQueue};

use super::projectile::Projectile;

pub struct ProjectileSystem {
    command_queue: Rc<RefCell<CommandQueue>>,
    gl: Rc<Context>,
    projectiles: HashMap<Uuid, Projectile>,
}

impl ProjectileSystem {
    pub fn new(gl: &Rc<Context>, command_queue: &Rc<RefCell<CommandQueue>>) -> ProjectileSystem {
        Self {
            command_queue: Rc::clone(command_queue),
            gl: Rc::clone(gl),
            projectiles: HashMap::new(),
        }
    }

    pub fn spawn_projectile(&mut self, transform: Mat4, velocity: Vec3) {
        debug!("Spawning projectile {transform}, {velocity}");
        let proj = Projectile::new(&self.gl, transform, velocity).unwrap();
        self.projectiles.insert(proj.id, proj);
    }

    pub fn remove_projectile(&mut self, id: Uuid) {
        self.projectiles.remove(&id);
    }

    pub fn tick(&mut self, dt: f32) {
        for (id, projectile) in &mut self.projectiles {
            projectile.tick(dt);
            if projectile.lifetime < 0.0 {
                self.command_queue
                    .borrow_mut()
                    .enqueue(crate::command_queue::Command::RemoveProjectile { id: *id });
            }
        }
    }

    pub fn render(&mut self, cam: &Camera) {
        for projectile in &mut self.projectiles.values_mut() {
            projectile.render(cam);
        }
    }
}
