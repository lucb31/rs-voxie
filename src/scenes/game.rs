use crate::{
    cameras::{camera::CameraController, thirdpersoncam::ThirdPersonCam},
    collision::system_voxel_world_collisions,
    command_queue::{Command, CommandQueue},
    input::InputState,
    logic::GameContext,
    renderer::ECSRenderer,
    systems::{
        gun::system_gun_fire,
        physics::{Transform, system_movement},
        player::{
            Player, render_player_ui, spawn_player, system_player_mouse_control,
            system_player_movement,
        },
        projectiles::{spawn_projectile, system_lifetime, system_projectile_collisions},
        skybox::spawn_skybox,
        voxels::system_voxel_world_growth,
    },
    voxels::{CHUNK_SIZE, VoxelWorld, VoxelWorldRenderer, generators::noise3d::Noise3DGenerator},
};
use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc};

use glam::Vec3;
use glow::HasContext;
use hecs::World;
use imgui::Ui;
use log::info;

use crate::{cameras::camera::Camera, scenes::Scene};

const INITIAL_WORLD_SIZE: usize = 4;

pub struct GameScene {
    gl: Rc<glow::Context>,
    voxel_renderer: VoxelWorldRenderer,
    ecs: World,
    ecs_renderer: ECSRenderer,
    // TODO: Probably no longer need to wrap in refcell
    world: Rc<RefCell<VoxelWorld>>,
    context: Rc<RefCell<GameContext>>,

    command_queue: Rc<RefCell<CommandQueue>>,

    camera: Rc<RefCell<Camera>>,
    camera_controller: Box<dyn CameraController>,
}

impl GameScene {
    pub fn new(
        gl: &Rc<glow::Context>,
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

        let command_queue = Rc::new(RefCell::new(CommandQueue::new()));
        let generator = Arc::new(Noise3DGenerator::new(CHUNK_SIZE));
        let world = Rc::new(RefCell::new(VoxelWorld::new(INITIAL_WORLD_SIZE, generator)));
        let voxel_renderer = VoxelWorldRenderer::new(gl)?;

        let mut ecs = World::new();
        spawn_player(&mut ecs, Vec3::splat(50.0));
        spawn_skybox(&mut ecs);

        Ok(Self {
            camera,
            camera_controller: Box::new(camera_controller),
            command_queue: Rc::clone(&command_queue),
            context,
            ecs,
            gl: Rc::clone(gl),
            ecs_renderer: ECSRenderer::new(gl)?,
            voxel_renderer,
            world,
        })
    }

    fn process_command_queue(&mut self) {
        for cmd in self.command_queue.borrow_mut().iter() {
            match cmd {
                Command::SpawnProjectile {
                    transform,
                    velocity,
                } => {
                    spawn_projectile(&mut self.ecs, transform, velocity);
                }
            }
        }
    }
}

impl Scene for GameScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        self.voxel_renderer.render_ui(ui);
        render_player_ui(&mut self.ecs, ui);
        self.world.borrow_mut().render_ui(ui);
    }

    fn get_title(&self) -> String {
        "Game".to_string()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    fn tick(&mut self, dt: f32) {
        // Entity lifetime (as early as possible to avoid simulating dead entities)
        system_lifetime(&mut self.ecs, dt);

        self.voxel_renderer.tick(dt, &self.camera.borrow().position);
        self.world.borrow_mut().tick();
        self.context.borrow_mut().tick();

        system_player_mouse_control(
            &mut self.ecs,
            &mut self.context.borrow_mut().input_state.borrow_mut(),
        );
        system_player_movement(
            &mut self.ecs,
            &self.context.borrow().input_state.borrow(),
            dt,
            &self.world.borrow(),
        );
        system_gun_fire(&mut self.ecs, &mut self.command_queue.borrow_mut(), dt);
        system_movement(&mut self.ecs, dt);
        // System camera controller
        {
            let mut query = self.ecs.query::<(&Player, &Transform)>();

            let (_entity, (_player, transform)) =
                query.iter().next().expect("No player found to follow");
            self.camera_controller
                .tick(dt, &mut self.camera.borrow_mut(), &transform.0);
        }

        let collision_events = system_voxel_world_collisions(&mut self.ecs, &self.world.borrow());
        system_projectile_collisions(
            &mut self.ecs,
            &mut self.world.borrow_mut(),
            &collision_events,
        );
        system_voxel_world_growth(&mut self.world.borrow_mut(), &self.camera.borrow().position);
        self.process_command_queue();
    }

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let cam = self.camera.borrow();
        self.voxel_renderer.render(&cam, &self.world.borrow());
        self.ecs_renderer.render(&mut self.ecs, &cam);
    }

    fn start(&mut self) {
        info!("Starting game scene...");
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }
}
