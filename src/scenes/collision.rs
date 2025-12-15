use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;

use crate::{
    cameras::camera::Camera,
    collision::{CollisionInfo, iter_sphere_collision},
    cube::CubeRenderer,
    meshes::sphere::SphereMesh,
    octree::IAabb,
    scenes::metrics::SimpleMovingAverage,
    scenes::{Renderer, Scene},
    voxels::{CHUNK_SIZE, VoxelWorld},
};

/// Used to debug & visualize collision tests
pub struct CollisionScene {
    camera: Rc<RefCell<Camera>>,
    sphere: SphereMesh,
    cube_renderer: CubeRenderer,
    world: Rc<RefCell<VoxelWorld>>,

    gl: Rc<glow::Context>,

    // Metrics / UI
    collisions: Vec<CollisionInfo>,
    sma_collision_check_time: SimpleMovingAverage,
    render_cubes: bool,
    render_sphere: bool,
    render_collision_points: bool,

    // DEBUG
    // Only test collision if sphere collider moved
    last_tested_position: Vec3,
    // Pool of spheres to visualize collision points
    collision_spheres: Vec<SphereMesh>,
}

impl CollisionScene {
    pub fn new(gl: &Rc<glow::Context>) -> Result<CollisionScene, Box<dyn Error>> {
        let mut camera = Camera::new();
        camera.position = Vec3::new(0.0, 3.0, -15.0);
        camera.set_rotation(
            Quat::from_rotation_y(-90f32.to_radians()) * Quat::from_rotation_x(-10f32.to_radians()),
        );

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS);
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let world = Rc::new(RefCell::new(VoxelWorld::new_cubic(1)));
        let mut cube_renderer = CubeRenderer::new(gl, world.clone())?;
        cube_renderer.color = Vec3::new(0.0, 1.0, 0.0);
        let mut sphere = SphereMesh::new(gl)?;
        sphere.position = Vec3::new(-2.5, 0.0, -0.1);
        sphere.radius = 0.49;
        sphere.color = Vec3::new(0.0, 0.0, 1.0);

        // Instantiate pool of spheres that will be used to visualize collision points
        let mut collision_spheres = Vec::with_capacity(4);
        for _i in 0..4 {
            let mut s = SphereMesh::new(gl)?;
            s.position = Vec3::ONE * -1000.0;
            s.color = Vec3::new(1.0, 0.0, 0.0);
            collision_spheres.push(s);
        }

        Ok(Self {
            collision_spheres,
            last_tested_position: Vec3::ONE * -999.0,
            collisions: Vec::with_capacity(4),
            sma_collision_check_time: SimpleMovingAverage::new(100),
            cube_renderer,
            sphere,
            camera: Rc::new(RefCell::new(camera)),
            world,
            gl: Rc::clone(gl),
            render_cubes: true,
            render_sphere: true,
            render_collision_points: true,
        })
    }
}

impl Scene for CollisionScene {
    fn get_title(&self) -> String {
        "Collision Test".to_string()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn tick(&mut self, dt: f32) {
        let camera_fov = IAabb::new(
            &IVec3::ZERO,
            self.world.borrow().get_size() * CHUNK_SIZE * 2,
        );
        self.cube_renderer.tick(dt, &camera_fov);
        // Update sphere
        if self.last_tested_position != self.sphere.position {
            self.collisions = iter_sphere_collision(
                &self.world.borrow(),
                self.sphere.position,
                self.sphere.radius,
            )
            .collect();
            self.last_tested_position = self.sphere.position;
            // Update collision points
            for i in 0..self.collision_spheres.len() {
                if self.collisions.len() > i {
                    self.collision_spheres[i].position = self.collisions[i].contact_point;
                } else {
                    self.collision_spheres[i].position = Vec3::ONE * -1000.0;
                }
            }
        }
    }

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        if self.render_cubes {
            self.cube_renderer.render(&self.camera.borrow_mut());
        }
        if self.render_sphere {
            self.sphere.render(&self.camera.borrow_mut());
        }
        if self.render_collision_points {
            for sphere in &mut self.collision_spheres {
                sphere.render(&self.camera.borrow_mut());
            }
        }
    }

    fn render_ui(&mut self, ui: &mut imgui::Ui) {
        ui.window("Collisions")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .position([400.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Current hits: {}", self.collisions.len(),));
                ui.text(format!(
                    "Avg time for collision check: {:.1}micro-s",
                    self.sma_collision_check_time.get()
                ));
                ui.separator();
                ui.text("Cube position");
                ui.slider(
                    "x",
                    -1.0,
                    CHUNK_SIZE as f32 + 2.0,
                    &mut self.sphere.position.x,
                );
                ui.slider(
                    "y",
                    -1.0,
                    CHUNK_SIZE as f32 + 2.0,
                    &mut self.sphere.position.y,
                );
                ui.slider(
                    "z",
                    -2.5,
                    CHUNK_SIZE as f32 + 2.0,
                    &mut self.sphere.position.z,
                );
                ui.separator();
                ui.checkbox("Render Cubes", &mut self.render_cubes);
                ui.checkbox("Render sphere", &mut self.render_sphere);
                ui.checkbox("Render Contact points", &mut self.render_collision_points);
            });
    }

    fn start(&mut self) {}
}
