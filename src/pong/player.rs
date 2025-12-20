use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use hecs::World;
use winit::keyboard::KeyCode;

use crate::{
    collision::ColliderBody,
    input::InputState,
    meshes::objmesh::ObjMesh,
    renderer::{Mesh, RenderMeshHandle, ecs_renderer::MESH_CUBE, shader::Shader},
    systems::physics::{Transform, Velocity},
};

pub struct PongPlayer;
struct PlayerMovement {
    speed: f32,
}

pub fn spawn_player(world: &mut World, position: Vec3) {
    let scale = Vec3::new(0.1, 1.0, 1.0);
    world.spawn((
        PongPlayer,
        Transform(Mat4::from_scale_rotation_translation(
            scale,
            Quat::IDENTITY,
            position,
        )),
        Velocity(Vec3::ZERO),
        RenderMeshHandle(MESH_CUBE),
        PlayerMovement { speed: 5.0 },
        ColliderBody::AabbCollider { scale },
    ));
}

/// Calculate player velocity based on keyboard input and collide_and_slide algorithm
/// Integration of velocity is done in general movement system
pub fn system_pong_movement(world: &mut World, input: &InputState, dt: f32) {
    for (_entity, (transform, velocity, movement)) in
        world.query_mut::<(&Transform, &mut Velocity, &PlayerMovement)>()
    {
        // Parse inputs
        let mut input_velocity = Vec3::ZERO;
        if input.is_key_pressed(&KeyCode::KeyW) {
            input_velocity += Vec3::Y;
        }
        if input.is_key_pressed(&KeyCode::KeyS) {
            input_velocity -= Vec3::Y;
        }
        if input.is_key_pressed(&KeyCode::KeyA) {
            input_velocity -= Vec3::X;
        }
        if input.is_key_pressed(&KeyCode::KeyD) {
            input_velocity += Vec3::X;
        }
        if input_velocity.length_squared() < 1e-4 {
            velocity.0 = Vec3::ZERO;
        } else {
            input_velocity *= movement.speed * dt;
            // TODO: Collision
            let collision_adjusted_velocity = input_velocity;
            // * dt will be applied again in movement system
            velocity.0 = collision_adjusted_velocity / dt;
        }
    }
}

//pub fn system_player_collision(world: &mut World) -> Vec<CollisionEvent> {
//    let mut query = world
//        .query::<(&Transform, &ColliderBody)>()
//        .without::<&PongPlayer>();
// Iterate over all unique pairs
//    for  in 0..colliders.len() {
//        for j in (i + 1)..colliders.len() {
//            let (entity_a, (transform_a, collider_a)) = colliders[i];
//            let (entity_b, (transform_b, collider_b)) = colliders[j];
//            let center_a = transform_a.0.w_axis.xyz();
//            let center_b = transform_b.0.w_axis.xyz();
//            let collision_info: Option<CollisionInfo> = match collider_a {
//                ColliderBody::SphereCollider { radius: radius_a } => match collider_b {
//                    ColliderBody::SphereCollider { radius: radius_b } => {
//                        get_sphere_sphere_collision_info(center_a, *radius_a, center_b, *radius_b)
//                    }
//                    ColliderBody::AabbCollider { scale: scale_b } => {
//                        let aabb_b = AABB::from_center_and_scale(&center_b, scale_b);
//                        get_sphere_aabb_collision_info(&center_a, *radius_a, &aabb_b)
//                    }
//                },
//                ColliderBody::AabbCollider { scale: scale_a } => {
//                    let aabb_a = AABB::from_center_and_scale(&center_a, scale_a);
//                    match collider_b {
//                        ColliderBody::SphereCollider { radius: radius_b } => {
//                            get_sphere_aabb_collision_info(&center_b, *radius_b, &aabb_a)
//                        }
//                        ColliderBody::AabbCollider { scale: scale_b } => {
//                            let aabb_b = AABB::from_center_and_scale(&center_b, scale_b);
//                            match aabb_a.intersects(&aabb_b) {
//                                true => Some(CollisionInfo {
//                                    normal: Vec3::ZERO,
//                                    contact_point: center_a,
//                                    distance: 0.0,
//                                }),
//                                false => None,
//                            }
//                        }
//                    }
//                }
//            };
//            if let Some(info) = collision_info {
//                all_collisions.push(CollisionEvent {
//                    info,
//                    a: entity_a,
//                    b: Some(entity_b),
//                });
//            }
//        }
//    }
//    all_collisions
//}

pub fn mesh_cube(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(gl, "assets/shaders/cube.vert", "assets/shaders/quad.frag")?;

    // Load vertex data from mesh
    let mut mesh = ObjMesh::new();
    mesh.load("assets/cube.obj").expect("Could not load mesh");
    let vertex_buffers = mesh.get_vertex_buffers();
    // NOTE: /3 because we have 3 coordinates per vertex
    let vertex_count = vertex_buffers.position_buffer.len() / 3;
    let positions_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.position_buffer);
    let normals_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.normal_buffer);
    let tex_coords_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.tex_coord_buffer);
    unsafe {
        // Setup vertex & index array and buffer
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        // Buffer position data
        let positions_vbo = gl.create_buffer().expect("Cannot create buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positions_bytes, gl::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(0, 3, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 0);
        // Buffer normal data
        let normals_vbo = gl
            .create_buffer()
            .expect("Cannot create buffer for normals");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normals_bytes, gl::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(1, 3, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 1);
        // Buffer texture coordinate data
        let tex_coords_vbo = gl.create_buffer().expect("Cannot create buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(tex_coords_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, tex_coords_bytes, gl::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(3, 2, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 3);
        gl.bind_buffer(gl::ARRAY_BUFFER, None);

        Ok(Mesh::new(shader, vao, vertex_count as i32))
    }
}
