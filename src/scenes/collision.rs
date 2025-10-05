use std::{error::Error, rc::Rc};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;

use crate::{
    camera::Camera,
    cube::CubeRenderer,
    meshes::sphere::SphereMesh,
    octree::IAabb,
    scene::{Renderer, Scene},
    voxel::CHUNK_SIZE,
    world::VoxelWorld,
};

/// Used to debug & visualize collision tests
pub struct CollisionScene {
    camera: Camera,
    sphere: SphereMesh,
    cube_renderer: CubeRenderer,
    world: Rc<VoxelWorld>,

    gl: Rc<glow::Context>,
}

impl CollisionScene {
    pub fn new(gl: Rc<glow::Context>) -> Result<CollisionScene, Box<dyn Error>> {
        let mut camera = Camera::new();
        camera.position = Vec3::new(-10.0, 10.0, -30.0);
        camera.set_rotation(
            Quat::from_rotation_y(-100f32.to_radians())
                * Quat::from_rotation_x(-25f32.to_radians()),
        );

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS);
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let world = Rc::new(VoxelWorld::new_cubic(2));
        let mut cube_renderer = CubeRenderer::new(gl.clone(), world.clone())?;
        cube_renderer.color = Vec3::new(0.0, 1.0, 0.0);
        let mut sphere = SphereMesh::new(gl.clone())?;
        sphere.position = Vec3::new(-5.0, 0.0, 0.0);

        Ok(Self {
            cube_renderer,
            sphere,
            camera,
            world,
            gl,
        })
    }
}

impl Scene for CollisionScene {
    fn get_title(&self) -> String {
        "Collision Test".to_string()
    }

    fn get_main_camera(&mut self) -> &mut crate::camera::Camera {
        &mut self.camera
    }

    fn get_stats(&self) -> crate::benchmark::SceneStats {
        todo!()
    }

    fn tick(&mut self, dt: f32, gl: &glow::Context) {
        let camera_fov = IAabb::new(&IVec3::ZERO, self.world.get_size() * CHUNK_SIZE * 2);
        self.cube_renderer.tick(dt, &camera_fov);
        self.camera.tick(dt);
        self.sphere.position.x += 1.0 * dt;
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.cube_renderer.render(gl, &self.camera);
        self.sphere.render(gl, &self.camera);
    }

    fn render_ui(&self, ui: &mut imgui::Ui) {}

    fn start(&mut self) {}
}
