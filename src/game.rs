use crate::scene::Renderer;
use std::error::Error;

use glam::{Quat, Vec3};
use glow::HasContext;
use noise::{NoiseFn, Perlin};

use crate::{
    camera::Camera,
    cube::{CubeMesh, CubeRenderer},
    octree::WorldTree,
    scene::Scene,
};

pub struct GameScene {
    camera: Camera,
    cube_renderer: CubeRenderer,
    world: WorldTree<CubeMesh>,
}

impl GameScene {
    pub fn new(gl: &glow::Context) -> Result<GameScene, Box<dyn Error>> {
        let mut camera = Camera::new();
        camera.position = Vec3::new(58.0, 37.0, 53.0);
        camera.set_rotation(
            Quat::from_rotation_y(45f32.to_radians()) * Quat::from_rotation_x(-25f32.to_radians()),
        );
        let world: WorldTree<CubeMesh> = generate_world(64)?;
        let mut cube_renderer = CubeRenderer::new(gl)?;

        let cubes = world.get_all_depth_first();
        cube_renderer.update_batches(gl, &cubes)?;

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
            world,
            cube_renderer,
        })
    }
}

// TODO: Does not render cubes correcly. Weird faces
impl Scene for GameScene {
    fn get_title(&self) -> String {
        todo!()
    }

    fn get_main_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn tick(&mut self, dt: f32) {
        // println!("TICK. Need to query region");
    }

    fn destroy(&mut self, gl: &glow::Context) {
        self.cube_renderer.destroy(gl);
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.cube_renderer.render(gl, &self.camera);
    }

    fn start(&mut self) {
        println!("Starting game scene...");
    }

    fn get_stats(&self) -> crate::benchmark::SceneStats {
        todo!()
    }
}

fn generate_world(initial_size: usize) -> Result<WorldTree<CubeMesh>, Box<dyn Error>> {
    let mut world = WorldTree::new(initial_size);
    println!("Generating world size {initial_size}");
    // TUNING
    const SEED: u32 = 99;
    let scale = 0.03;
    let perlin = Perlin::new(SEED);
    let max_height = (initial_size - 1) as f64;

    let mut nodes = 0;
    for x in 0..initial_size as i32 {
        let fx = x as f64 * scale;
        for z in 0..initial_size as i32 {
            let fz = z as f64 * scale;
            let noise_val = perlin.get([fx, fz]);
            let max_y = ((noise_val + 1.0) * (max_height / 2.0)).floor() as i32;
            for y in 0..max_y {
                let mut cube = CubeMesh::new()?;
                cube.position = Vec3::new(x as f32, y as f32, z as f32);
                cube.color = Vec3::new(0.0, 1.0, 0.0);
                world.insert(x, y, z, cube);
                nodes += 1;
            }
        }
    }
    println!("Generated {nodes} nodes");
    Ok(world)
}
