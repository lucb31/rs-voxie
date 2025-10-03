use std::{error::Error, time::Instant};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;
use imgui::Ui;

use crate::{
    benchmark::SceneStats,
    camera::Camera,
    cube::{self, CubeRenderer},
    octree::IAabb,
    quadmesh,
    voxel::CHUNK_SIZE,
    world::VoxelWorld,
};

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

    world: VoxelWorld,
    cube_renderer: CubeRenderer,

    cube_count: usize,
    frame_count: u32,
}

impl BenchmarkScene {
    pub fn new(gl: &glow::Context, world_size: usize) -> Result<BenchmarkScene, Box<dyn Error>> {
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

        // Setup cube world
        let world = VoxelWorld::new_cubic(world_size);
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
            cube_count: world_size * world_size * world_size,
            world,
            cube_renderer,
            title: "Unnamed scene".to_string(),
            camera,
            last: now,
            start: now,
            renderers,
            frame_count: 0,
        })
    }
}

impl Scene for BenchmarkScene {
    fn render_ui(&self, ui: &mut Ui) {}

    fn render(&mut self, gl: &glow::Context) {
        // On first tick: Load cubes
        if self.frame_count == 0 {
            self.cube_renderer
                .update_batches(
                    gl,
                    &self.world.query_region_chunks(&IAabb::new(
                        &IVec3::ZERO,
                        self.world.get_size() * CHUNK_SIZE * 2,
                    )),
                )
                .expect("Failed to update batches");
        }
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
            self.cube_count as u32,
        )
    }
}
