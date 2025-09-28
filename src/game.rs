use crate::{octree::AABB, scene::Renderer, voxel::Voxel};
use std::error::Error;

use glam::{Quat, Vec3};
use glow::HasContext;
use imgui::Ui;
use noise::{NoiseFn, Perlin};

use crate::{camera::Camera, cube::CubeRenderer, octree::WorldTree, scene::Scene};

pub struct GameScene {
    camera: Camera,
    cube_renderer: CubeRenderer,
    world: WorldTree<Voxel>,

    // Region in which the camera will 'see'
    camera_fov: AABB,
}

const CAMERA_BB_SIZE: f32 = 128.0;
const CAMERA_FOV_SIZE: f32 = 256.0;

impl GameScene {
    pub fn new(gl: &glow::Context) -> Result<GameScene, Box<dyn Error>> {
        let mut camera = Camera::new();
        camera.position = Vec3::new(44.0, 50.0, 50.0);
        camera.set_rotation(
            Quat::from_rotation_y(45f32.to_radians()) * Quat::from_rotation_x(-25f32.to_radians()),
        );
        let world: WorldTree<Voxel> = generate_world(256)?;
        let mut cube_renderer = CubeRenderer::new(gl)?;
        cube_renderer.color = Vec3::new(0.0, 1.0, 0.0);

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let origin = Vec3::ZERO;
        let mut instance = Self {
            cube_renderer,
            camera_fov: AABB::new(&origin, CAMERA_FOV_SIZE),
            camera,
            world,
        };

        instance.update_batches(gl)?;

        Ok(instance)
    }

    // Update camera FoV and pass cubes within FoV to cube renderer
    fn update_batches(&mut self, gl: &glow::Context) -> Result<(), Box<dyn Error>> {
        self.camera_fov = AABB::new(&self.camera.position, CAMERA_FOV_SIZE);
        let visible_cubes = self.world.query_region(&self.camera_fov);
        self.cube_renderer.update_batches(gl, &visible_cubes)?;
        Ok(())
    }
}

impl Scene for GameScene {
    fn render_ui(&self, ui: &mut Ui) {
        ui.window("Game Debug")
            .position([600.0, 200.0], imgui::Condition::FirstUseEver)
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("FoV: ({:?}", self.camera_fov));
                ui.separator();
            });
    }

    fn get_title(&self) -> String {
        todo!()
    }

    fn get_main_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn tick(&mut self, dt: f32, gl: &glow::Context) {
        // Check if camera is close to boundaries
        let camera_bb = AABB::new(&self.camera.position, CAMERA_BB_SIZE);
        if !self.camera_fov.contains(&camera_bb) {
            println!(
                "Camera BB {:?} reached camera FoV threshold {:?}, time to adjust",
                camera_bb, self.camera_fov
            );
            self.update_batches(gl).expect("Could not update batches");
        }
        self.camera.tick(dt);
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

// NOTE: Required until rendering becomes smarter
// Currently if we allow for same height as width and depth, we just
// generate a bunch of cubes, that are not visible and should not be drawn
// once rendering is smarter
const HEIGHT_LIMIT: i32 = 32;

fn generate_world(initial_size: usize) -> Result<WorldTree<Voxel>, Box<dyn Error>> {
    let mut world = WorldTree::new(initial_size, Vec3::ZERO);
    println!("Generating world size {initial_size}");
    // TUNING
    const SEED: u32 = 99;
    let scale = 0.03;
    let perlin = Perlin::new(SEED);

    let mut nodes = 0;
    let half = initial_size as i32 / 2;
    let max_height = HEIGHT_LIMIT.min(half - 1) as f64;
    for x in -half + 1..half {
        let fx = x as f64 * scale;
        for z in -half + 1..half {
            let fz = z as f64 * scale;
            let noise_val = perlin.get([fx, fz]);
            let max_y = ((noise_val + 1.0) * (max_height / 2.0)).floor() as i32;
            for y in 0..max_y {
                let mut voxel = Voxel::new();
                voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                world.insert(voxel.position, voxel);
                nodes += 1;
            }
        }
    }
    println!("World generation produced {nodes} nodes");
    Ok(world)
}

// Used for testing purposes
fn generate_cubic_world(initial_size: usize) -> WorldTree<Voxel> {
    let mut world = WorldTree::new(initial_size, Vec3::ZERO);
    println!("Generating cubic world size {initial_size}");
    let mut nodes = 0;
    let half = initial_size as i32 / 2;
    for x in -half + 1..half + 1 {
        for z in -half + 1..half + 1 {
            for y in -half + 1..half + 1 {
                let mut voxel = Voxel::new();
                voxel.position = Vec3::new(x as f32, y as f32, z as f32);
                world.insert(voxel.position, voxel);
                nodes += 1;
            }
        }
    }
    println!("World generation produced {nodes} nodes");
    world
}
