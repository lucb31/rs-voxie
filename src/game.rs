use crate::{octree::AABB, scene::Renderer, voxel::Voxel, world::generate_world};
use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Quat, Vec3};
use glow::HasContext;
use imgui::Ui;

use crate::{camera::Camera, cube::CubeRenderer, octree::WorldTree, scene::Scene};

pub struct GameScene {
    camera: Camera,
    cube_renderer: CubeRenderer,
    world: WorldTree<Rc<RefCell<Voxel>>>,

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
        let world = generate_world(256)?;
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
    fn render_ui(&self, _ui: &mut Ui) {}

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
