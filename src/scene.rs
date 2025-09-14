use std::time::Instant;

use gl::{BACK, CCW, CULL_FACE};
use glam::Vec3;
use glow::HasContext;

use crate::{camera::Camera, cube::CubeMesh, quadmesh};

pub trait Mesh {
    fn render(&self, gl: &glow::Context, cam: &Camera);
    // TODO: This should not be part of the mesh trait.
    fn tick(&mut self, dt: f32);
    fn destroy(&self, gl: &glow::Context);
}

pub struct Scene {
    last: Instant,
    pub camera: Camera,
    meshes: Vec<Box<dyn Mesh>>,
}

impl Scene {
    pub fn new(gl: &glow::Context) -> Scene {
        let now = Instant::now();
        let camera = Camera::new();

        let cube = CubeMesh::new(gl);
        let mut plane = CubeMesh::new(gl);
        plane.position = Vec3::new(25.0, -5.0, 25.0);
        plane.scale = Vec3::new(50.0, 0.1, 50.0);
        let mut quad = quadmesh::QuadMesh::new(gl);
        quad.scale = Vec3::new(0.1, 0.1, 1.0);
        quad.position = Vec3::new(-0.5, -0.5, 0.0);
        quad.color = Vec3::new(0.0, 1.0, 0.0);
        let meshes: Vec<Box<dyn Mesh>> = vec![Box::new(cube), Box::new(plane), Box::new(quad)];
        Self {
            camera,
            last: now,
            meshes,
        }
    }

    pub fn render(&mut self, gl: &glow::Context) {
        // FIX: Currently processing and rendering will be done in one step
        // MAYBE this does not have to be mutable once we separate the steps
        self.process();

        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
            gl.enable(CULL_FACE);
            gl.cull_face(BACK);
            gl.front_face(CCW);
        }

        for mesh in &self.meshes {
            mesh.render(gl, &self.camera);
        }
    }

    pub fn process(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        debug_assert!(dt > 0.0);
        for mesh in &mut self.meshes {
            mesh.tick(dt);
        }
        self.last = now;
    }

    pub fn destroy(&self, gl: &glow::Context) {
        for mesh in &self.meshes {
            mesh.destroy(gl);
        }
    }
}
