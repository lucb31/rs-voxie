use std::{error::Error, rc::Rc};

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::{cameras::camera::Camera, meshes::sphere::SphereMesh, scenes::Renderer};

pub struct Projectile {
    transform: Mat4,
    velocity: Vec3,
    mesh: SphereMesh,
}

impl Projectile {
    pub fn new(
        gl: &Rc<glow::Context>,
        transform: Mat4,
        velocity: Vec3,
    ) -> Result<Projectile, Box<dyn Error>> {
        let mesh = SphereMesh::new(gl)?;
        Ok(Self {
            transform,
            velocity,
            mesh,
        })
    }

    pub fn tick(&mut self, dt: f32) {
        let vec = self.velocity * dt;
        self.transform.w_axis += Vec4::new(vec.x, vec.y, vec.z, 0.0);
        let position = self.transform.w_axis.xyz();
        self.mesh.position = position;
    }

    pub fn render(&mut self, cam: &Camera) {
        self.mesh.render(cam);
    }
}
