use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use hecs::World;
use log::debug;
use winit::keyboard::KeyCode;

use crate::{
    collision::ColliderBody,
    input::InputState,
    renderer::{
        RenderMeshHandle,
        ecs_renderer::{MESH_PLAYER, RenderColor},
    },
    systems::{
        gun::Gun,
        physics::{LocalTransform, Parent},
    },
    voxels::{VoxelCollider, VoxelWorld},
};

use crate::systems::physics::Transform;
use crate::systems::physics::Velocity;

pub mod squid;

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

pub fn spawn_player(world: &mut hecs::World, position: Vec3) -> hecs::Entity {
    // Root entity: controls movement, mouse rotation
    let root = world.spawn((
        Player,
        LocalTransform {
            local: Mat4::from_translation(position),
        },
        Transform(Mat4::from_translation(position)),
        Velocity(Vec3::ZERO),
        VoxelCollider,
        ColliderBody::SphereCollider { radius: 0.5 },
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

    // Mesh entity: child of root, static 180° Y rotation
    world.spawn((
        LocalTransform {
            local: Mat4::from_rotation_y(std::f32::consts::PI),
        },
        Transform(Mat4::from_rotation_y(std::f32::consts::PI)),
        RenderMeshHandle(MESH_PLAYER),
        RenderColor(Vec3::splat(0.85)),
        Parent(root),
    ));

    root
}

pub fn system_player_mouse_control(world: &mut World, input: &InputState) {
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
                ui.slider("Mouse sensitivity", 0.001, 0.003, &mut mouse.sensitivity);
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
        &ColliderBody,
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
    collider: &ColliderBody,
) -> Vec3 {
    if depth >= MAX_COLLIDE_BOUNCES {
        return Vec3::ZERO;
    }
    debug_assert!(vel.is_finite());
    match collider {
        ColliderBody::SphereCollider { radius } => {
            let dist = vel.length() + SKIN_WIDTH;
            let vel_normalized = vel.normalize();
            let collision_test =
                voxel_world.query_sphere_cast(pos, radius - SKIN_WIDTH, vel_normalized, dist);
            if let Some(collision) = collision_test {
                let mut snap_to_surface =
                    vel_normalized * (collision.penetration_depth - SKIN_WIDTH);
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
        ColliderBody::AabbCollider { .. } => {
            todo!(
                "Missing implementation: Voxel world collide and slide with aabb collider character controller"
            )
        }
        ColliderBody::CapsuleCollider { .. } => {
            todo!(
                "Missing implementation: Voxel world collide and slide with capsule collider character controller"
            )
        }
    }
}
