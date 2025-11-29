use crate::{
    cameras::{camera::CameraController, thirdpersoncam::ThirdPersonCam},
    input::InputState,
    meshes::quadmesh::QuadMesh,
    player::Player,
    scene::Renderer,
    voxel::CHUNK_SIZE,
    voxels::{generators::noise3d::Noise3DGenerator, voxel_renderer::VoxelWorldRenderer},
    world::VoxelWorld,
};
use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc};

use glam::{Quat, Vec3};
use glow::HasContext;
use imgui::Ui;
use log::info;

use crate::{cameras::camera::Camera, scene::Scene};

pub struct GameContext {
    pub input_state: Rc<RefCell<InputState>>,
}
impl GameContext {
    pub fn new(input_state: Rc<RefCell<InputState>>) -> GameContext {
        Self { input_state }
    }
}

const INITIAL_WORLD_SIZE: usize = 16;

pub struct GameScene {
    voxel_renderer: VoxelWorldRenderer,
    world: Rc<RefCell<VoxelWorld>>,
    context: Rc<RefCell<GameContext>>,
    player: Player,

    camera: Rc<RefCell<Camera>>,
    camera_controller: Box<dyn CameraController>,

    world_boundary_planes: [QuadMesh; 3],
}

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
        let world = Rc::new(RefCell::new(VoxelWorld::new(INITIAL_WORLD_SIZE, generator)));
        let voxel_renderer = VoxelWorldRenderer::new(gl.clone(), world.clone())?;
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
            context,
            voxel_renderer,
            world_boundary_planes: planes,
            player,
            world,
        })
    }
}

impl Scene for GameScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        self.voxel_renderer.render_ui(ui);
        self.player.render_ui(ui);
    }

    fn get_title(&self) -> String {
        "Game".to_string()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    // TODO: stop passing around gls
    fn tick(&mut self, dt: f32, _gl: &glow::Context) {
        self.player.tick(dt);
        self.voxel_renderer.tick(dt, &self.camera.borrow().position);
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
        self.voxel_renderer.render(&cam);
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
