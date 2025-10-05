use std::{error::Error, rc::Rc, time::Instant};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;

use crate::{
    camera::Camera,
    collision::get_sphere_aabb_collision_info,
    cube::CubeRenderer,
    meshes::sphere::SphereMesh,
    octree::{AABB, IAabb},
    scene::{Renderer, Scene},
    util::SimpleMovingAverage,
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

    // Metrics / UI
    collision_hits: i32,
    sma_collision_check_time: SimpleMovingAverage,
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

        let world = Rc::new(VoxelWorld::new_cubic(1));
        let mut cube_renderer = CubeRenderer::new(gl.clone(), world.clone())?;
        cube_renderer.color = Vec3::new(0.0, 1.0, 0.0);
        let mut sphere = SphereMesh::new(gl.clone())?;
        sphere.position = Vec3::new(-5.0, 0.0, 0.0);

        Ok(Self {
            collision_hits: 0,
            sma_collision_check_time: SimpleMovingAverage::new(100),
            cube_renderer,
            sphere,
            camera,
            world,
            gl,
        })
    }

    fn sphere_collision_test(&mut self) {
        let start = Instant::now();
        // BB test
        let sphere_box_region_f = AABB::new_center(&self.sphere.position, self.sphere.radius * 2.0);
        let sphere_box_region_i = IAabb::from(&sphere_box_region_f);
        let voxels = self.world.query_region_voxels(&sphere_box_region_i);
        // Collision test
        let mut colliding = 0;
        for voxel in &voxels {
            let vox_collider = voxel.get_collider();
            let collision_info = get_sphere_aabb_collision_info(
                &self.sphere.position,
                self.sphere.radius,
                &vox_collider,
            );
            if collision_info.is_some() {
                colliding += 1;
            }
        }
        self.collision_hits = colliding;
        self.sma_collision_check_time
            .add(start.elapsed().as_secs_f32() * 1e6);
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
        // Update sphere
        self.sphere.position.x += 1.0 * dt;
        self.sphere_collision_test();
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.cube_renderer.render(gl, &self.camera);
        self.sphere.render(gl, &self.camera);
    }

    fn render_ui(&self, ui: &mut imgui::Ui) {
        ui.window("Collisions")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([1200.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Current hits: {}", self.collision_hits,));
                ui.text(format!(
                    "Avg time for collision check: {:.1}micro-s",
                    self.sma_collision_check_time.get()
                ));
            });
    }

    fn start(&mut self) {}
}
