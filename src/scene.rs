use std::{error::Error, time::Instant};

use glam::{Quat, Vec3};
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

        let mut cube_center = CubeMesh::new(gl);
        cube_center.color = Vec3::new(1.0, 0.0, 0.0);
        cube_center.position = Vec3::new(0.0, 0.5, 0.0);
        let mut cube_bottom_left = CubeMesh::new(gl);
        cube_bottom_left.color = Vec3::new(0.0, 0.0, 1.0);
        cube_bottom_left.position = Vec3::new(-20.0, 0.5, -20.0);
        let mut cube_bottom_right = CubeMesh::new(gl);
        cube_bottom_right.color = Vec3::new(0.0, 0.0, 1.0);
        cube_bottom_right.position = Vec3::new(20.0, 0.5, -20.0);
        let mut cube_top_right = CubeMesh::new(gl);
        cube_top_right.color = Vec3::new(0.0, 0.0, 1.0);
        cube_top_right.position = Vec3::new(20.0, 0.5, 20.0);
        let mut cube_top_left = CubeMesh::new(gl);
        cube_top_left.color = Vec3::new(0.0, 0.0, 1.0);
        cube_top_left.position = Vec3::new(-20.0, 0.5, 20.0);
        let mut ground_quad = quadmesh::QuadMesh::new(gl)?;
        ground_quad.scale = Vec3::new(200.0, 200.0, 1.0);
        ground_quad.rotation = Quat::from_rotation_x(-90f32.to_radians());
        let meshes: Vec<Box<dyn Mesh>> = vec![
            // Test cube
            Box::new(cube_center),
            Box::new(cube_top_left),
            Box::new(cube_top_right),
            Box::new(cube_bottom_left),
            Box::new(cube_bottom_right),
            // Quad to render ground grid
            Box::new(ground_quad),
        ];

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
