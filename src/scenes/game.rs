use crate::{
    cameras::{camera::CameraController, thirdpersoncam::ThirdPersonCam},
    collision::{VoxelCollider, system_voxel_world_collisions},
    command_queue::{Command, CommandQueue},
    ecs::{Lifetime, Projectile, Transform, Velocity, system_lifetime, system_movement},
    input::InputState,
    logic::GameContext,
    meshes::quadmesh::QuadMesh,
    octree::IAabb,
    player::Player,
    projectiles::projectile_system::ProjectileSystem,
    renderer::{ECSRenderer, MESH_PROJECTILE, RenderMeshHandle},
    voxels::{
        CHUNK_SIZE, VoxelKind, VoxelWorld, VoxelWorldRenderer,
        generators::noise3d::Noise3DGenerator,
    },
};
use std::{cell::RefCell, error::Error, rc::Rc, sync::Arc};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;
use hecs::World;
use imgui::Ui;
use log::{debug, info};

use crate::{
    cameras::camera::Camera,
    scenes::{Renderer, Scene},
};

const INITIAL_WORLD_SIZE: usize = 4;

pub struct GameScene {
    gl: Rc<glow::Context>,
    voxel_renderer: VoxelWorldRenderer,
    ecs: World,
    ecs_renderer: ECSRenderer,
    world: Rc<RefCell<VoxelWorld>>,
    context: Rc<RefCell<GameContext>>,

    player: Player,
    projectile_system: ProjectileSystem,
    command_queue: Rc<RefCell<CommandQueue>>,

    camera: Rc<RefCell<Camera>>,
    camera_controller: Box<dyn CameraController>,

    world_boundary_planes: [QuadMesh; 6],
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
        let voxel_renderer = VoxelWorldRenderer::new(gl, world.clone())?;
        let mut player = Player::new(gl, &context, &world, &command_queue)?;
        player.position = Vec3::ONE * 50.0;

        // Setup world boundary planes planes
        let mut plane_min_x = QuadMesh::new(gl)?;
        plane_min_x.scale = Vec3::ONE * 1e3;
        plane_min_x.rotation = Quat::from_rotation_x(-90f32.to_radians());
        plane_min_x.color = Vec3::X;
        let mut plane_min_y = QuadMesh::new(gl)?;
        plane_min_y.scale = Vec3::ONE * 1e3;
        plane_min_y.rotation = Quat::from_rotation_y(90f32.to_radians());
        plane_min_y.color = Vec3::Y;
        let mut plane_min_z = QuadMesh::new(gl)?;
        plane_min_z.scale = Vec3::ONE * 1e3;
        plane_min_z.rotation = Quat::from_rotation_z(-90f32.to_radians());
        plane_min_z.color = Vec3::Z;
        let mut plane_max_x = QuadMesh::new(gl)?;
        plane_max_x.scale = Vec3::ONE * 1e3;
        plane_max_x.rotation = Quat::from_rotation_x(90f32.to_radians());
        plane_max_x.color = Vec3::X;
        plane_max_x.position = Vec3::new(0.0, 1e3, 0.0);
        let mut plane_max_y = QuadMesh::new(gl)?;
        plane_max_y.scale = Vec3::ONE * 1e3;
        plane_max_y.rotation = Quat::from_rotation_y(-90f32.to_radians());
        plane_max_y.color = Vec3::Y;
        plane_max_y.position = Vec3::new(1e3, 0.0, 0.0);
        let mut plane_max_z = QuadMesh::new(gl)?;
        plane_max_z.scale = Vec3::ONE * 1e3;
        plane_max_z.rotation = Quat::from_rotation_y(-180f32.to_radians());
        plane_max_z.color = Vec3::Z;
        plane_max_z.position = Vec3::new(0.0, 0.0, 1e3);
        let planes = [
            plane_min_x,
            plane_min_y,
            plane_min_z,
            plane_max_x,
            plane_max_y,
            plane_max_z,
        ];

        Ok(Self {
            camera,
            camera_controller: Box::new(camera_controller),
            command_queue: Rc::clone(&command_queue),
            context,
            ecs: World::new(),
            gl: Rc::clone(gl),
            player,
            ecs_renderer: ECSRenderer::new(gl)?,
            voxel_renderer,
            projectile_system: ProjectileSystem::new(gl, &command_queue),
            world,
            world_boundary_planes: planes,
        })
    }

    /// Removes all voxels in a radius around the player.
    /// This is only used for demonstration purposes
    fn demo_voxel_player_collision(&mut self) {
        let collider_size = 2;
        let collider = IAabb::new(
            &IVec3::new(
                (self.player.position.x - collider_size as f32 / 2.0).round() as i32,
                (self.player.position.y - collider_size as f32 / 2.0).round() as i32,
                (self.player.position.z - collider_size as f32 / 2.0).round() as i32,
            ),
            collider_size,
        );
        let chunks = self
            .world
            .borrow_mut()
            .query_region_chunks_with_init(&collider);
        let mut voxels_removed = 0;
        for chunk in &chunks {
            for voxel in chunk.voxel_slice() {
                if voxel.position.distance_squared(self.player.position)
                    < (collider_size * collider_size) as f32
                {
                    // Within radius
                    let mut new_voxel = *voxel;
                    new_voxel.kind = VoxelKind::Air;
                    chunk.insert(
                        &IVec3::new(
                            voxel.position.x as i32,
                            voxel.position.y as i32,
                            voxel.position.z as i32,
                        ),
                        new_voxel,
                    );
                    voxels_removed += 1;
                }
            }
        }
        if voxels_removed > 0 {
            debug!(
                "Removed {} colliding voxels from {} chunks",
                voxels_removed,
                chunks.len()
            );
        }
    }

    fn process_command_queue(&mut self) {
        for cmd in self.command_queue.borrow_mut().iter() {
            match cmd {
                Command::SpawnProjectile {
                    transform,
                    velocity,
                } => {
                    self.ecs.spawn((
                        Transform(transform),
                        Velocity(velocity),
                        VoxelCollider::SphereCollider { radius: 0.25 },
                        Projectile,
                        RenderMeshHandle(MESH_PROJECTILE),
                        Lifetime(2.0),
                    ));
                    debug!("Projectile spawned {:?}, {}", transform, velocity);
                }
                // TODO: Deprecate
                Command::RemoveProjectile { id } => self.projectile_system.remove_projectile(id),
            }
        }
    }
}

impl Scene for GameScene {
    fn render_ui(&mut self, ui: &mut Ui) {
        self.voxel_renderer.render_ui(ui);
        self.player.render_ui(ui);
        self.world.borrow_mut().render_ui(ui);
    }

    fn get_title(&self) -> String {
        "Game".to_string()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    fn tick(&mut self, dt: f32) {
        self.player.tick(dt);
        self.voxel_renderer.tick(dt, &self.camera.borrow().position);
        self.camera_controller.tick(
            dt,
            &mut self.camera.borrow_mut(),
            &self.player.get_transform(),
        );
        self.demo_voxel_player_collision();
        self.world.borrow_mut().tick();
        self.process_command_queue();
        // TODO: Deprecate / refactor projectile system
        // self.projectile_system.tick(dt);
        self.context.borrow_mut().tick();

        // Entity lifetime (as early as possible to avoid simulating dead entities)
        system_lifetime(&mut self.ecs, dt);
        system_movement(&mut self.ecs, dt);

        // Process projectiles
        let collision_events = system_voxel_world_collisions(&mut self.ecs, &self.world.borrow());
        // Projectile collisions
        for collision in collision_events {
            if self.ecs.get::<&Projectile>(collision.a).is_ok() {
                // Projectile involved
                self.ecs
                    .despawn(collision.a)
                    .expect("Unable to remove projectile");
                debug!("Projectile hit the world!! Removing");
                // TODO: Explosion radius
            }
        }
    }

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        let cam = self.camera.borrow();
        self.player.render(&cam);
        self.voxel_renderer.render(&cam);
        // Render utility planes to visualize world boundaries
        for plane in &mut self.world_boundary_planes {
            plane.render(&cam);
        }
        // TODO: Remove all projectile_system
        // self.projectile_system.render(&cam);

        self.ecs_renderer.render(&mut self.ecs, &cam);
    }

    fn start(&mut self) {
        info!("Starting game scene...");
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }
}
