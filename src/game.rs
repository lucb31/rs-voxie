use crate::{
    cameras::{camera::CameraController, thirdpersoncam::ThirdPersonCam},
    meshes::quadmesh::QuadMesh,
    octree::IAabb,
    player::Player,
    scene::Renderer,
    voxel::CHUNK_SIZE,
    voxels::generators::noise3d::Noise3DGenerator,
    world::VoxelWorld,
};
use std::{cell::RefCell, collections::HashSet, error::Error, rc::Rc, sync::Arc};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;
use imgui::Ui;
use log::{debug, info};
use winit::{event::MouseButton, keyboard::KeyCode};

use crate::{cameras::camera::Camera, cube::CubeRenderer, scene::Scene};

pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_delta: (f64, f64),
}

impl InputState {
    pub fn new() -> InputState {
        let keys_pressed = HashSet::<KeyCode>::new();
        let mouse_buttons_pressed = HashSet::<MouseButton>::new();
        Self {
            keys_pressed,
            mouse_buttons_pressed,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn key_pressed(&mut self, code: KeyCode) {
        self.keys_pressed.insert(code);
    }
    pub fn key_released(&mut self, code: &KeyCode) {
        self.keys_pressed.remove(code);
    }
    pub fn mouse_button_pressed(&mut self, button: MouseButton) {
        self.mouse_buttons_pressed.insert(button);
        // WARN: Interims fix to reset delta when middle mouse button is clicked
        // Otherwise the inputs get buffered and applied once first pressed
        if self.is_mouse_button_pressed(&MouseButton::Middle) {
            self.mouse_delta = (0.0, 0.0);
        }
    }
    pub fn mouse_button_released(&mut self, button: &MouseButton) {
        self.mouse_buttons_pressed.remove(button);
    }
    pub fn mouse_moved(&mut self, delta: (f64, f64)) {
        self.mouse_delta.0 += delta.0;
        self.mouse_delta.1 += delta.1;
    }
    // Just and interims fix until we've figured out how multiple components
    // can consume the delta
    pub fn get_and_reset_mouse_moved(&mut self) -> (f64, f64) {
        let res = (self.mouse_delta.0, self.mouse_delta.1);
        self.mouse_delta = (0.0, 0.0);
        res
    }
    pub fn is_mouse_button_pressed(&self, btn: &MouseButton) -> bool {
        self.mouse_buttons_pressed.get(btn).is_some()
    }
}

pub struct GameContext {
    pub input_state: Rc<RefCell<InputState>>,
}
impl GameContext {
    pub fn new(input_state: Rc<RefCell<InputState>>) -> GameContext {
        Self { input_state }
    }
}

pub struct GameScene {
    cube_renderer: CubeRenderer,
    world: Rc<RefCell<VoxelWorld>>,
    context: Rc<RefCell<GameContext>>,
    player: Player,

    // Region in which the camera will 'see'
    camera_fov: IAabb,
    camera: Rc<RefCell<Camera>>,
    camera_controller: Box<dyn CameraController>,

    world_boundary_planes: [QuadMesh; 3],
}

// Determines size of 'smaller' camera bb that checks if we need to update FoV
const CAMERA_BB_VOXELS: i32 = 48;
// How many voxels the camera can see in one direction
const CAMERA_FOV_VOXELS: i32 = 64;

impl GameScene {
    pub fn new(
        gl: Rc<glow::Context>,
        input_state: Rc<RefCell<InputState>>,
    ) -> Result<GameScene, Box<dyn Error>> {
        // Camera setup
        let camera = Rc::new(RefCell::new(Camera::new()));
        let camera_controller = ThirdPersonCam::new();

        // Setup context
        let context_instance = GameContext::new(input_state);
        let context = Rc::new(RefCell::new(context_instance));
        // Prepare rendering
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let generator = Arc::new(Noise3DGenerator::new(CHUNK_SIZE));
        let world = Rc::new(RefCell::new(VoxelWorld::new(16, generator)));
        let mut cube_renderer = CubeRenderer::new(gl.clone(), world.clone())?;
        cube_renderer.color = Vec3::new(0.0, 1.0, 0.0);
        let mut player = Player::new(gl.clone(), camera.clone(), context.clone(), world.clone())?;
        player.position = Vec3::ONE * 50.0;

        // Setup world boundary planes planes
        let mut plane_x = QuadMesh::new(gl.clone())?;
        plane_x.scale = Vec3::ONE * 1e3;
        plane_x.rotation = Quat::from_rotation_x(-90f32.to_radians());
        plane_x.color = Vec3::X;
        let mut plane_y = QuadMesh::new(gl.clone())?;
        plane_y.scale = Vec3::ONE * 1e3;
        plane_y.rotation = Quat::from_rotation_y(90f32.to_radians());
        plane_y.color = Vec3::Y;
        let mut plane_z = QuadMesh::new(gl.clone())?;
        plane_z.scale = Vec3::ONE * 1e3;
        plane_z.rotation = Quat::from_rotation_z(-90f32.to_radians());
        plane_z.color = Vec3::Z;
        let planes = [plane_x, plane_y, plane_z];

        Ok(Self {
            camera,
            camera_controller: Box::new(camera_controller),
            // Doesnt matter, we just need to initialize, we'll update once initialized in
            // update_batches
            camera_fov: IAabb::new(&IVec3::ZERO, 1),
            context,
            cube_renderer,
            world_boundary_planes: planes,
            player,
            world,
        })
    }
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut chars = s.chars().rev().enumerate();
    while let Some((i, c)) = chars.next() {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}

impl Scene for GameScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        ui.window("Cubes")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([300.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!(
                    "Rendered cubes: {}",
                    format_with_commas(self.cube_renderer.get_instance_count() as u64)
                ));
            });
        ui.window("Player")
            .size([300.0, 300.0], imgui::Condition::FirstUseEver)
            .position([600.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Position: {}", self.player.position));
                ui.text(format!("Velocity: {}", self.player.velocity));
            });
    }

    fn get_title(&self) -> String {
        "Game".to_string()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    // TODO: stop passing around gls
    fn tick(&mut self, dt: f32, _gl: &glow::Context) {
        // Check if camera is close to boundaries and we need to update FoV
        // NOTE: Would like to move to camera tick. But need to figure out how to set cube renderer
        // dirty then
        {
            let camera = self.camera.borrow();
            let camera_bb = IAabb::new(
                &IVec3::new(
                    camera.position.x as i32 - CAMERA_BB_VOXELS,
                    camera.position.y as i32 - CAMERA_BB_VOXELS,
                    camera.position.z as i32 - CAMERA_BB_VOXELS,
                ),
                (CAMERA_BB_VOXELS * 2) as usize,
            );
            if !self.camera_fov.contains(&camera_bb) {
                // Update camera FoV
                self.camera_fov = IAabb::new(
                    &IVec3::new(
                        camera.position.x as i32 - CAMERA_FOV_VOXELS,
                        camera.position.y as i32 - CAMERA_FOV_VOXELS,
                        camera.position.z as i32 - CAMERA_FOV_VOXELS,
                    ),
                    (CAMERA_FOV_VOXELS * 2) as usize,
                );
                // Tell cube renderer to update
                self.cube_renderer.is_dirty = true;
            }
        }

        self.player.tick(dt);
        self.cube_renderer.tick(dt, &self.camera_fov);
        self.camera_controller.tick(
            dt,
            &mut self.camera.borrow_mut(),
            &self.player.get_transform(),
        );
    }

    // TODO: stop passing around gls
    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.player.render();
        let cam = self.camera.borrow();
        self.cube_renderer.render(gl, &cam);
        // Render utility planes to visualize world boundaries
        for plane in &mut self.world_boundary_planes {
            plane.render(gl, &cam);
        }
    }

    fn start(&mut self) {
        info!("Starting game scene...");
    }

    fn get_stats(&self) -> crate::benchmark::SceneStats {
        todo!()
    }
}
