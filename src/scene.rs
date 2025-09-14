use std::{error::Error, time::Instant};

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
    pub fn new(gl: &glow::Context) -> Result<Scene, Box<dyn Error>> {
        let now = Instant::now();
        let camera = Camera::new();

        let mut cube = CubeMesh::new(gl);
        cube.color = Vec3::new(0.0, 0.0, 1.0);
        cube.position = Vec3::new(0.0, 0.5, 0.0);
        let mut plane = CubeMesh::new(gl);
        plane.scale = Vec3::new(50.0, 0.1, 50.0);
        let mut quad = quadmesh::QuadMesh::new(gl)?;
        quad.scale = Vec3::new(0.1, 0.1, 1.0);
        quad.position = Vec3::new(-0.5, -0.5, 0.0);
        quad.color = Vec3::new(0.0, 1.0, 0.0);
        let meshes: Vec<Box<dyn Mesh>> = vec![Box::new(plane), Box::new(cube), Box::new(quad)];

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        Ok(Self {
            camera,
            last: now,
            meshes,
        })
    }

    pub fn render(&mut self, gl: &glow::Context) {
        // FIX: Currently processing and rendering will be done in one step
        // MAYBE this does not have to be mutable once we separate the steps
        self.process();

        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
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
