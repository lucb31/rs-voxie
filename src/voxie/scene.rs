use crate::{
    cameras::{camera::CameraController, thirdpersoncam::ThirdPersonCam},
    command_queue::{Command, CommandQueue},
    config::{RESOLUTION_HEIGHT, RESOLUTION_WIDTH},
    input::InputState,
    renderer::{ECSRenderer, Mesh},
    scenes::scene::BaseScene,
    systems::{
        gun::system_gun_fire,
        physics::{
            Transform, hierarchy_cache::HierarchyCache, system_movement_with_hierarchy_nodes,
        },
        projectiles::{spawn_projectile, system_lifetime, system_projectile_collisions},
        skybox::{fog_mesh, quad_mesh, spawn_skybox},
        voxels::system_voxel_world_growth,
    },
    voxels::{
        CHUNK_SIZE, VoxelWorld, VoxelWorldRenderer, generators::noise3d::Noise3DGenerator,
        system_voxel_world_collisions,
    },
    voxie::player::{
        Player, render_player_ui, system_player_mouse_control, system_player_movement,
    },
};
use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc, time::Duration};

use glam::{Mat4, Vec3};
use glow::{HasContext, NativeFramebuffer, NativeTexture};
use hecs::World;
use imgui::Ui;
use log::info;

use crate::{cameras::camera::Camera, scenes::GuiScene};

use super::{
    game_context::GameContext,
    player::{
        squid::{spawn_squid, system_squid_velocity_tilt},
        system_player_keyboard_control,
    },
};

const INITIAL_WORLD_SIZE: usize = 4;

pub struct GameScene {
    ecs: World,
    hierarchy_cache: HierarchyCache,

    // TODO: Probably no longer need to wrap in refcell
    world: Rc<RefCell<VoxelWorld>>,
    context: Rc<RefCell<GameContext>>,

    command_queue: Rc<RefCell<CommandQueue>>,

    camera: Rc<RefCell<Camera>>,
    camera_controller: Box<dyn CameraController>,

    // Rendering
    ecs_renderer: ECSRenderer,
    voxel_renderer: VoxelWorldRenderer,
    geometry_fbo: NativeFramebuffer,
    post_process_quad: Mesh,
    first_pass_texture: NativeTexture,
    first_pass_depth_texture: NativeTexture,
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

        // Initialize game mechanics
        let command_queue = Rc::new(RefCell::new(CommandQueue::new()));
        let generator = Arc::new(Noise3DGenerator::new(CHUNK_SIZE));
        let world = Rc::new(RefCell::new(VoxelWorld::new(INITIAL_WORLD_SIZE, generator)));

        // Initialize ECS world
        let mut ecs = World::new();
        spawn_squid(&mut ecs, Vec3::splat(50.0));
        //spawn_skybox(&mut ecs);

        // Setup rendering
        let post_process_quad = fog_mesh(gl)?;
        let voxel_renderer = VoxelWorldRenderer::new(gl)?;
        unsafe {
            let width = RESOLUTION_WIDTH as i32;
            let height = RESOLUTION_HEIGHT as i32;

            // Setup geometry pass framebuffer
            let geometry_fbo = gl.create_framebuffer()?;
            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(geometry_fbo));
            // Setup frame color texture
            let frame_color_tex = gl.create_texture()?;
            gl.bind_texture(gl::TEXTURE_2D, Some(frame_color_tex));
            gl.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::RGB as i32,
                width,
                height,
                0,
                gl::RGB,
                gl::UNSIGNED_BYTE,
                None,
            );
            gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
            // Attach color texture to framebuffer
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                Some(frame_color_tex),
                0,
            );

            // Setup depth & stencil buffer
            let ds_texture = gl.create_texture()?;
            gl.bind_texture(gl::TEXTURE_2D, Some(ds_texture));
            gl.tex_image_2d(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH24_STENCIL8 as i32,
                width,
                height,
                0,
                gl::DEPTH_STENCIL,
                gl::UNSIGNED_INT_24_8,
                None,
            );
            // Sample only depth values into texture
            gl.tex_parameter_i32(
                gl::TEXTURE_2D,
                gl::DEPTH_STENCIL_TEXTURE_MODE,
                gl::DEPTH_COMPONENT as i32,
            );
            gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
            gl.tex_parameter_i32(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            // Attach stencil texture to framebuffer
            gl.framebuffer_texture_2d(
                gl::FRAMEBUFFER,
                gl::DEPTH_STENCIL_ATTACHMENT,
                gl::TEXTURE_2D,
                Some(ds_texture),
                0,
            );

            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            Ok(Self {
                first_pass_depth_texture: ds_texture,
                geometry_fbo,
                first_pass_texture: frame_color_tex,
                post_process_quad,
                camera,
                camera_controller: Box::new(camera_controller),
                command_queue: Rc::clone(&command_queue),
                context,
                ecs,
                hierarchy_cache: HierarchyCache::new(),
                ecs_renderer: ECSRenderer::new(gl)?,
                voxel_renderer,
                world,
            })
        }
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

impl BaseScene for GameScene {
    fn get_title(&self) -> String {
        "Voxie".to_string()
    }

    fn tick(&mut self, dt: f32) {
        // Entity lifetime (as early as possible to avoid simulating dead entities)
        system_lifetime(&mut self.ecs, dt);

        self.context.borrow_mut().tick();

        system_player_mouse_control(&mut self.ecs, &self.context.borrow().input_state.borrow());
        system_player_keyboard_control(&mut self.ecs, &self.context.borrow().input_state.borrow());
        system_player_movement(&mut self.ecs, dt, &self.world.borrow());
        system_squid_velocity_tilt(&mut self.ecs, dt);
        system_gun_fire(&mut self.ecs, &mut self.command_queue.borrow_mut(), dt);
        system_movement_with_hierarchy_nodes(&mut self.ecs, dt, &mut self.hierarchy_cache);

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
        if self.context.borrow().current_frame % 60 == 0 {
            // Check for world expansion once a second
            system_voxel_world_growth(&mut self.world.borrow_mut(), &self.camera.borrow().position);
        }
        self.world.borrow_mut().receive_chunks();
        self.process_command_queue();
    }

    fn start(&mut self) {
        info!("Starting game scene...");
    }

    fn get_world(&self) -> Option<&World> {
        None
    }
}

impl GuiScene for GameScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        self.voxel_renderer.render_ui(ui);
        render_player_ui(&mut self.ecs, ui);
        self.world.borrow_mut().render_ui(ui);
    }

    fn render(&mut self, gl: &glow::Context, _dt: Duration) {
        // Prepare rendering
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        // 1. Main render pass
        unsafe {
            gl.bind_framebuffer(gl::FRAMEBUFFER, Some(self.geometry_fbo));
            gl.clear_color(0.0, 0.411, 0.58, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let cam = self.camera.borrow();
        self.voxel_renderer.render(&cam, &self.world.borrow());
        self.ecs_renderer.render_camera(
            &self.ecs,
            &cam,
            self.context.borrow().start_time.elapsed().as_secs_f32(),
        );

        // 2. Render pass for post-processing
        unsafe {
            gl.bind_framebuffer(gl::FRAMEBUFFER, None);
            gl.clear_color(0.0, 0.411, 0.58, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT);

            // Wireframe mode
            //gl.polygon_mode(gl::FRONT_AND_BACK, gl::LINE);
        }
        let shader = &mut self.post_process_quad.shader;
        shader.use_program();
        let vao = self.post_process_quad.vao;
        let count = self.post_process_quad.vertex_count;
        unsafe {
            gl.disable(gl::DEPTH_TEST);
            gl.bind_vertex_array(Some(vao));
            // Bind first pass color texture
            gl.active_texture(gl::TEXTURE0);
            gl.bind_texture(gl::TEXTURE_2D, Some(self.first_pass_texture));
            // Bind first pass depth texture
            gl.active_texture(gl::TEXTURE1);
            gl.bind_texture(gl::TEXTURE_2D, Some(self.first_pass_depth_texture));
            gl.active_texture(gl::TEXTURE0);
            gl.draw_elements(glow::TRIANGLES, count, gl::UNSIGNED_INT, 0);
            gl.bind_vertex_array(None);
        }
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }
}
