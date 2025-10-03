use crate::{octree::IAabb, scene::Renderer, world::VoxelWorld};
use std::{
    error::Error,
    rc::Rc,
    sync::{Mutex, mpsc::Receiver},
    time::Instant,
};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;
use imgui::Ui;

use crate::{camera::Camera, cube::CubeRenderer, scene::Scene};

pub struct GameScene {
    camera: Camera,
    cube_renderer: CubeRenderer,
    world: Rc<VoxelWorld>,

    // Region in which the camera will 'see'
    camera_fov: IAabb,
}

// Determines size of 'smaller' camera bb that checks if we need to update FoV
const CAMERA_BB_VOXELS: i32 = 64;
// How many voxels the camera can see in one direction
const CAMERA_FOV_VOXELS: i32 = 128;

impl GameScene {
    pub fn new(gl: Rc<glow::Context>) -> Result<GameScene, Box<dyn Error>> {
        let mut camera = Camera::new();
        camera.position = Vec3::new(44.0, 50.0, 50.0);
        camera.set_rotation(
            Quat::from_rotation_y(45f32.to_radians()) * Quat::from_rotation_x(-25f32.to_radians()),
        );

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let world = Rc::new(VoxelWorld::new(16));
        let mut cube_renderer = CubeRenderer::new(gl, world.clone())?;
        cube_renderer.color = Vec3::new(0.0, 1.0, 0.0);

        Ok(Self {
            cube_renderer,
            // Doesnt matter, we just need to initialize, we'll update once initialized in
            // update_batches
            camera_fov: IAabb::new(&IVec3::ZERO, 1),
            camera,
            world,
        })
    }
}

impl Scene for GameScene {
    fn render_ui(&self, _ui: &mut Ui) {}

    fn get_title(&self) -> String {
        todo!()
    }

    fn get_main_camera(&mut self) -> &mut Camera {
        &mut self.camera
    }

    fn tick(&mut self, dt: f32, gl: &glow::Context) {
        // Check if camera is close to boundaries and we need to update FoV
        // TODO: Move to camera tick
        let camera_bb = IAabb::new(
            &IVec3::new(
                self.camera.position.x as i32 - CAMERA_BB_VOXELS,
                self.camera.position.y as i32 - CAMERA_BB_VOXELS,
                self.camera.position.z as i32 - CAMERA_BB_VOXELS,
            ),
            (CAMERA_BB_VOXELS * 2) as usize,
        );
        if !self.camera_fov.contains(&camera_bb) {
            // Update camera FoV & tell cube renderer to update
            self.camera_fov = IAabb::new(
                &IVec3::new(
                    self.camera.position.x as i32 - CAMERA_FOV_VOXELS,
                    self.camera.position.y as i32 - CAMERA_FOV_VOXELS,
                    self.camera.position.z as i32 - CAMERA_FOV_VOXELS,
                ),
                (CAMERA_FOV_VOXELS * 2) as usize,
            );
            self.cube_renderer.is_dirty = true;
        }
        self.cube_renderer.tick(dt, &self.camera_fov);
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
