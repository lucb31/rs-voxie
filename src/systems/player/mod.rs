use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use glow::HasContext;
use hecs::World;
use log::debug;
use winit::keyboard::KeyCode;

use crate::{
    input::InputState,
    meshes::objmesh::ObjMesh,
    renderer::{MESH_PROJECTILE, Mesh, RenderMeshHandle, shader::Shader},
    systems::gun::Gun,
    voxels::VoxelCollider,
    voxels::VoxelWorld,
};

use crate::systems::physics::Transform;
use crate::systems::physics::Velocity;

pub struct Player;
struct MousePanConfig {
    pub sensitivity: f32,
    pub last_mouse_position: (f32, f32),
    pub yaw: f32,
    pub pitch: f32,
}
struct PlayerMovement {
    pub speed: f32,
}

pub fn spawn_player(world: &mut World, position: Vec3) {
    world.spawn((
        Player,
        Transform(Mat4::from_translation(position)),
        Velocity(Vec3::ZERO),
        // TODO: Use player mesh
        RenderMeshHandle(MESH_PROJECTILE),
        VoxelCollider::SphereCollider { radius: 0.5 },
        MousePanConfig {
            last_mouse_position: (0.0, 0.0),
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        },
        PlayerMovement { speed: 15.0 },
        Gun {
            cooldown: 0.0,
            fire_rate: 2.5,
            triggered: false,
        },
    ));
}

pub fn system_player_mouse_control(world: &mut World, input: &mut InputState) {
    for (_entity, (transform, mouse_pan)) in
        world.query_mut::<(&mut Transform, &mut MousePanConfig)>()
    {
        let current_mouse_position = input.get_mouse_position_f32();
        let dx = mouse_pan.last_mouse_position.0 - current_mouse_position.0;
        let dy = mouse_pan.last_mouse_position.1 - current_mouse_position.1;
        mouse_pan.last_mouse_position = current_mouse_position;

        // Update yaw and pitch
        mouse_pan.yaw -= dx * mouse_pan.sensitivity;
        mouse_pan.pitch -= dy * mouse_pan.sensitivity;

        // Clamp pitch to [-89°, 89°] to prevent flipping
        let pitch_limit = std::f32::consts::FRAC_PI_2 - 0.01; // ~89.4°
        mouse_pan.pitch = mouse_pan.pitch.clamp(-pitch_limit, pitch_limit);

        let rotation = Quat::from_euler(glam::EulerRot::YXZ, mouse_pan.yaw, mouse_pan.pitch, 0.0);
        transform.0 = override_rotation(transform.0, rotation);
    }
}

fn override_rotation(mat: Mat4, rotation: Quat) -> Mat4 {
    let translation = mat.w_axis.truncate(); // extract translation
    let scale = Vec3::new(
        mat.x_axis.truncate().length(),
        mat.y_axis.truncate().length(),
        mat.z_axis.truncate().length(),
    ); // extract scale

    // Build new matrix with original scale & translation, but new rotation
    Mat4::from_scale_rotation_translation(scale, rotation, translation)
}

pub fn render_player_ui(world: &mut World, ui: &mut imgui::Ui) {
    for (_entity, (transform, velocity, mouse, movement)) in world.query_mut::<(
        &Transform,
        &Velocity,
        &mut MousePanConfig,
        &mut PlayerMovement,
    )>() {
        ui.window("Player")
            .size([300.0, 150.0], imgui::Condition::FirstUseEver)
            .position([600.0, 0.0], imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!("Position: {:.2}", transform.0.w_axis.xyz()));
                ui.text(format!("Velocity: {:.2}", velocity.0));
                ui.slider("Player speed", 5.0, 50.0, &mut movement.speed);
                ui.slider("Mouse sensitivity", 0.01, 0.03, &mut mouse.sensitivity);
            });
    }
}

/// Calculate player velocity based on keyboard input and collide_and_slide algorithm
/// Integration of velocity is done in general movement system
pub fn system_player_movement(
    world: &mut World,
    input: &InputState,
    dt: f32,
    voxel_world: &VoxelWorld,
) {
    for (_entity, (transform, velocity, movement, collider, gun)) in world.query_mut::<(
        &Transform,
        &mut Velocity,
        &PlayerMovement,
        &VoxelCollider,
        &mut Gun,
    )>() {
        // Parse inputs
        let mut input_velocity = Vec3::ZERO;
        let forward = (-transform.0.z_axis.xyz()).normalize();
        if input.is_key_pressed(&KeyCode::KeyW) {
            input_velocity += forward;
        }
        if input.is_key_pressed(&KeyCode::KeyS) {
            input_velocity -= forward;
        }
        if input.is_mouse_button_pressed(&winit::event::MouseButton::Left) {
            debug!("Gun fire requested");
            gun.triggered = true;
        }
        if input_velocity.length_squared() < 1e-4 {
            velocity.0 = Vec3::ZERO;
        } else {
            input_velocity *= movement.speed * dt;
            let collision_adjusted_velocity = collide_and_slide(
                input_velocity,
                transform.0.w_axis.xyz(),
                0,
                voxel_world,
                collider,
            );
            // * dt will be applied again in movement system
            velocity.0 = collision_adjusted_velocity / dt;
        }
    }
}

const MAX_COLLIDE_BOUNCES: u32 = 3;
const SKIN_WIDTH: f32 = 0.015;

/// Collide and slide algorithm. Basic version. Based on
/// https://www.youtube.com/watch?v=YR6Q7dUz2uk
fn collide_and_slide(
    vel: Vec3,
    pos: Vec3,
    depth: u32,
    voxel_world: &VoxelWorld,
    collider: &VoxelCollider,
) -> Vec3 {
    if depth >= MAX_COLLIDE_BOUNCES {
        return Vec3::ZERO;
    }
    debug_assert!(vel.is_finite());
    match collider {
        VoxelCollider::SphereCollider { radius } => {
            let dist = vel.length() + SKIN_WIDTH;
            let vel_normalized = vel.normalize();
            let collision_test =
                voxel_world.query_sphere_cast(pos, radius - SKIN_WIDTH, vel_normalized, dist);
            if let Some(collision) = collision_test {
                let mut snap_to_surface = vel_normalized * (collision.distance - SKIN_WIDTH);
                let leftover = vel - snap_to_surface;

                if snap_to_surface.length() <= SKIN_WIDTH {
                    snap_to_surface = Vec3::ZERO;
                }

                let leftover_length = leftover.length();
                let projection_normalized = leftover.project_onto(collision.normal).normalize();
                debug_assert!(projection_normalized.is_finite());
                let projection = projection_normalized * leftover_length;
                return snap_to_surface
                    + collide_and_slide(
                        projection,
                        pos + snap_to_surface,
                        depth + 1,
                        voxel_world,
                        collider,
                    );
            }
            vel
        }
    }
}

// Better than placing this randomly and having interdependencies between ecsrenderer and
// mesh implementations would be an asset manager that keeps track of meshes and allows registering
// / loading meshes
pub fn player_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/projectile.vert",
        "assets/shaders/sphere_rt.frag",
    )?;
    // Load vertex data from mesh
    let mut mesh = ObjMesh::new();
    mesh.load("assets/cube_github.obj")
        .expect("Could not load mesh");
    let vertex_positions = mesh.get_vertex_buffers().position_buffer;
    let vertex_bytes: &[u8] = bytemuck::cast_slice(&vertex_positions);
    unsafe {
        // Setup vertex & index array and buffer
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        // Bind vertex data
        let vbo = gl.create_buffer()?;
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(
            0,
            3,
            gl::FLOAT,
            false,
            3 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_array_attrib(vao, 0);
        Ok(Mesh::new(shader, vao, vertex_positions.len() as i32))
    }
}
