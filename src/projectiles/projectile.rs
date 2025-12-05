use std::{error::Error, rc::Rc};

use glam::{Mat4, Vec3, Vec4, Vec4Swizzles};

use crate::{cameras::camera::Camera, meshes::sphere::SphereMesh, scenes::Renderer};

pub struct Projectile {
    pub id: uuid::Uuid,
    transform: Mat4,
    velocity: Vec3,
    mesh: SphereMesh,
    pub lifetime: f32,
}

impl Projectile {
    pub fn new(
        gl: &Rc<glow::Context>,
        transform: Mat4,
        velocity: Vec3,
    ) -> Result<Projectile, Box<dyn Error>> {
        let mut mesh = SphereMesh::new(gl)?;
        mesh.radius = 0.25;
        mesh.color = Vec3::X;

        Ok(Self {
            id: uuid::Uuid::new_v4(),
            lifetime: 3.0,
            mesh,
            transform,
            velocity,
        })
    }

    pub fn tick(&mut self, dt: f32) {
        let vec = self.velocity * dt;
        self.transform.w_axis += Vec4::new(vec.x, vec.y, vec.z, 0.0);
        let position = self.transform.w_axis.xyz();
        self.mesh.position = position;
        self.lifetime -= dt;
    }

    pub fn render(&mut self, cam: &Camera) {
        self.mesh.render(cam);
    }
}
