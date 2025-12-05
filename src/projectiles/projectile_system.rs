use std::rc::Rc;

use glam::{Mat4, Vec3};
use glow::Context;
use log::debug;

use crate::cameras::camera::Camera;

use super::projectile::Projectile;

pub struct ProjectileSystem {
    gl: Rc<Context>,
    projectiles: Vec<Projectile>,
}

impl ProjectileSystem {
    pub fn new(gl: &Rc<Context>) -> ProjectileSystem {
        Self {
            gl: Rc::clone(gl),
            projectiles: vec![],
        }
    }

    pub fn spawn_projectile(&mut self, transform: Mat4, velocity: Vec3) {
        debug!("Spawning projectile {transform}, {velocity}");
        let proj = Projectile::new(&self.gl, transform, velocity).unwrap();
        self.projectiles.push(proj);
    }

    pub fn tick(&mut self, dt: f32) {
        for projectile in &mut self.projectiles {
            projectile.tick(dt);
        }
    }

    pub fn render(&mut self, cam: &Camera) {
        for projectile in &mut self.projectiles {
            projectile.render(cam);
        }
    }
}
