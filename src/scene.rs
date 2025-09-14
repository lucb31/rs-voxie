use std::{collections::HashSet, error::Error, time::Instant};

use glam::{Quat, Vec3};
use glow::HasContext;
use noise::{NoiseFn, Perlin};

use crate::{
    benchmark::SceneStats,
    camera::Camera,
    cube::{CubeMesh, CubeRenderer},
    quadmesh,
};

pub trait Renderer {
    fn render(&self, gl: &glow::Context, cam: &Camera);
    fn destroy(&self, gl: &glow::Context);
}

pub struct Scene {
    pub title: String,

    pub start: Instant,
    pub last: Instant,
    pub camera: Camera,
    // Rethink. We might not even need this
    renderers: Vec<Box<dyn Renderer>>,

    cube_renderer: CubeRenderer,

    cube_count: u32,
    frame_count: u32,
}

impl Scene {
    pub fn new(gl: &glow::Context) -> Result<Scene, Box<dyn Error>> {
        let now = Instant::now();
        let camera = Camera::new();

        // Quad to render ground grid
        let mut ground_quad = quadmesh::QuadMesh::new(gl)?;
        ground_quad.scale = Vec3::new(200.0, 200.0, 1.0);
        ground_quad.rotation = Quat::from_rotation_x(-90f32.to_radians());
        let renderers: Vec<Box<dyn Renderer>> = vec![Box::new(ground_quad)];
        let cube_renderer = CubeRenderer::new(gl)?;

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        Ok(Self {
            cube_count: 0,
            cube_renderer,
            title: "Unnamed scene".to_string(),
            camera,
            last: now,
            start: now,
            renderers,
            frame_count: 0,
        })
    }

    pub fn get_stats(&self) -> SceneStats {
        SceneStats::new(
            self.frame_count,
            self.start,
            self.last,
            self.title.to_string(),
            self.cube_count,
        )
    }

    pub fn add_cubes(&mut self, gl: &glow::Context, count: usize) -> Result<(), Box<dyn Error>> {
        let cubes = generate_cube_meshes_perlin(count)?;
        self.cube_renderer.update_batches(gl, &cubes)?;
        self.cube_count = cubes.len() as u32;
        Ok(())
    }

    pub fn render(&mut self, gl: &glow::Context) {
        // FIX: Currently processing and rendering will be done in one step
        // MAYBE this does not have to be mutable once we separate the steps
        self.process();

        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        for renderer in &self.renderers {
            renderer.render(gl, &self.camera);
        }
        self.cube_renderer.render(gl, &self.camera);
        self.frame_count += 1;
    }

    // Any physics logic will go here
    pub fn process(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        debug_assert!(dt > 0.0);
        self.last = now;
    }

    pub fn destroy(&self, gl: &glow::Context) {
        for mesh in &self.renderers {
            mesh.destroy(gl);
        }
    }
}

fn generate_cube_meshes_perlin(count: usize) -> Result<Vec<CubeMesh>, Box<dyn Error>> {
    let positions = generate_cube_positions(32, 16, 32, 0.3, 0.001, 48);
    let mut cubes = Vec::with_capacity(positions.len());
    for position in positions.iter().take(count) {
        let mut cube = CubeMesh::new()?;
        cube.position = Vec3::new(position.x as f32, position.y as f32, position.y as f32);
        println!("{:?}", cube.position);
        cube.color = Vec3::new(0.0, 1.0, 0.0);
        cubes.push(cube);
    }
    Ok(cubes)
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
struct Vec3i {
    x: i32,
    y: i32,
    z: i32,
}

fn generate_cube_positions(
    width: i32,
    height: i32,
    depth: i32,
    scale: f64,
    threshold: f64,
    seed: u32,
) -> HashSet<Vec3i> {
    let perlin = Perlin::new(seed);

    let mut cubes = HashSet::new();

    for x in 0..width {
        for y in 0..height {
            for z in 0..depth {
                let fx = x as f64 * scale;
                let fy = y as f64 * scale;
                let fz = z as f64 * scale;

                let noise_value = perlin.get([fx, fy, fz]);

                if noise_value > threshold {
                    cubes.insert(Vec3i { x, y, z });
                }
            }
        }
    }

    cubes
}
