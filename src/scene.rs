use std::{error::Error, time::Instant};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;
use imgui::Ui;
use noise::{NoiseFn, Perlin};

use crate::{benchmark::SceneStats, camera::Camera, cube::CubeRenderer, quadmesh, voxel::Voxel};

pub trait Renderer {
    fn render(&self, gl: &glow::Context, cam: &Camera);
    fn destroy(&self, gl: &glow::Context);
}

pub trait Scene {
    fn get_title(&self) -> String;
    fn get_main_camera(&mut self) -> &mut Camera;
    fn get_stats(&self) -> SceneStats;
    fn tick(&mut self, dt: f32, gl: &glow::Context);
    fn destroy(&mut self, gl: &glow::Context);
    fn render(&mut self, gl: &glow::Context);
    fn render_ui(&self, ui: &mut Ui);
    // Perform any initialization logic the scene might need
    fn start(&mut self);
}

pub struct BenchmarkScene {
    pub title: String,

    pub start: Instant,
    pub last: Instant,
    pub camera: Camera,
    // Rethink. We might not even need this
    renderers: Vec<Box<dyn Renderer>>,

    cube_renderer: CubeRenderer,

    cubes: Vec<Voxel>,
    frame_count: u32,
}

impl BenchmarkScene {
    pub fn new(gl: &glow::Context) -> Result<BenchmarkScene, Box<dyn Error>> {
        let now = Instant::now();
        let mut camera = Camera::new();
        camera.position = Vec3::new(58.0, 37.0, 53.0);
        camera.set_rotation(
            Quat::from_rotation_y(45f32.to_radians()) * Quat::from_rotation_x(-25f32.to_radians()),
        );

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
            cubes: vec![],
            cube_renderer,
            title: "Unnamed scene".to_string(),
            camera,
            last: now,
            start: now,
            renderers,
            frame_count: 0,
        })
    }

    pub fn add_cubes(&mut self, gl: &glow::Context, count: usize) -> Result<(), Box<dyn Error>> {
        println!("WARNING: Cube count currently not respected");
        self.cubes = generate_cube_slice(0, 16, 0, 16)?;
        self.cube_renderer.update_batches(gl, &self.cubes)?;
        Ok(())
    }
}

impl Scene for BenchmarkScene {
    fn render_ui(&self, ui: &mut Ui) {}

    fn render(&mut self, gl: &glow::Context) {
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

    fn tick(&mut self, dt: f32, gl: &glow::Context) {
        let now = Instant::now();
        self.last = now;
    }

    fn start(&mut self) {
        self.start = Instant::now();
    }

    fn destroy(&mut self, gl: &glow::Context) {
        for mesh in &self.renderers {
            mesh.destroy(gl);
        }
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn get_main_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn get_stats(&self) -> SceneStats {
        SceneStats::new(
            self.frame_count,
            self.start,
            self.last,
            self.title.to_string(),
            self.cubes.len() as u32,
        )
    }
}

const HEIGHT_MAP_SEED: u32 = 42;

// NOTE: To improve performance we could combine height map sampling
// loop with generating meshes.
// For now we'll separate just to keep it easier to understand
fn generate_cube_slice(
    xmin: i32,
    xmax: i32,
    ymin: i32,
    ymax: i32,
) -> Result<Vec<Voxel>, Box<dyn Error>> {
    println!("Generating cube slice [{xmin}..{xmax}][{ymin}..{ymax}]");
    debug_assert!(xmax > xmin);
    debug_assert!(ymax > ymin);
    // Dimensions
    let width = xmax - xmin;
    let height = ymax - ymin;
    // Helps to preallocate vector capacity
    let average_height = 16;
    let heights = generate_height_map(xmin, xmax, ymin, ymax);
    let mut cubes = Vec::with_capacity((width * height * average_height) as usize);
    for height_vector in heights.iter() {
        debug_assert!(height_vector.z >= 0);
        for z in 0..height_vector.z {
            let mut cube = Voxel::new();
            cube.position = IVec3::new(height_vector.x, z, height_vector.y);
            cubes.push(cube);
        }
    }
    Ok(cubes)
}

struct Vec3i {
    x: i32,
    y: i32,
    z: i32,
}

fn generate_height_map(xmin: i32, xmax: i32, ymin: i32, ymax: i32) -> Vec<Vec3i> {
    // TUNING
    let scale = 0.03;
    let perlin = Perlin::new(HEIGHT_MAP_SEED);
    let max_height = 10.0;

    let dim_x = xmax - xmin;
    let dim_y = ymax - ymin;
    debug_assert!(dim_x > 0);
    debug_assert!(dim_y > 0);
    let mut samples = Vec::with_capacity((dim_x * dim_y) as usize);
    for x in xmin..xmax {
        let fx = x as f64 * scale;
        for y in ymin..ymax {
            let fy = y as f64 * scale;
            let noise_value = (perlin.get([fx, fy]) * max_height + max_height).round();
            samples.push(Vec3i {
                x,
                y,
                z: noise_value as i32,
            });
        }
    }
    samples
}
