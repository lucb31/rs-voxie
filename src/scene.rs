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

        // Quad to render ground grid
        let mut ground_quad = quadmesh::QuadMesh::new(gl)?;
        ground_quad.scale = Vec3::new(200.0, 200.0, 1.0);
        ground_quad.rotation = Quat::from_rotation_x(-90f32.to_radians());
        let meshes: Vec<Box<dyn Mesh>> = vec![Box::new(ground_quad)];

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

    pub fn add_cubes(&mut self, gl: &glow::Context, count: usize) {
        let cubes = generate_cubes(count, gl);
        for cube in cubes {
            self.meshes.push(Box::new(cube));
        }
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

fn generate_cubes(count: usize, gl: &glow::Context) -> Vec<CubeMesh> {
    let mut cubes = Vec::with_capacity(count);
    // Compute the cube root and round up to get dimensions
    let size = (count as f64).cbrt().ceil() as usize;
    let spacing = 1.0; // Distance between cubes
    let mut placed = 0;
    for x in 0..size {
        for y in 0..size {
            for z in 0..size {
                if placed >= count {
                    return cubes;
                }
                let mut cube = CubeMesh::new(gl);
                cube.position =
                    Vec3::new(x as f32 * spacing, y as f32 * spacing, z as f32 * spacing);
                cube.color = Vec3::new(0.0, 1.0, 0.0);
                cubes.push(cube);
                placed += 1;
            }
        }
    }
    cubes
}
