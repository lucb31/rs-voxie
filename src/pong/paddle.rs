use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3, Vec4Swizzles};
use glow::HasContext;
use hecs::{Entity, World};

use crate::{
    collision::{ColliderBody, CollisionEvent},
    meshes::objmesh::ObjMesh,
    renderer::{Mesh, RenderMeshHandle, ecs_renderer::MESH_CUBE, shader::Shader},
    systems::physics::{Transform, Velocity},
};
pub struct PongPaddle {
    pub(super) speed: f32,
    pub(super) input_velocity: Vec3,
}

pub fn spawn_paddle(world: &mut World, position: Vec3) -> Entity {
    let scale = Vec3::new(0.1, 1.0, 1.0);
    world.spawn((
        Transform(Mat4::from_scale_rotation_translation(
            scale,
            Quat::IDENTITY,
            position,
        )),
        Velocity(Vec3::ZERO),
        RenderMeshHandle(MESH_CUBE),
        PongPaddle {
            speed: 1.0,
            input_velocity: Vec3::ZERO,
        },
        ColliderBody::AabbCollider { scale },
    ))
}

/// Calculate paddle velocity based on requested velocity and collide_and_slide algorithm
/// Integration of velocity is done in general movement system
pub fn system_paddle_movement(world: &mut World, collisions: &[CollisionEvent]) {
    for (entity, (transform, velocity, movement)) in
        world.query_mut::<(&Transform, &mut Velocity, &PongPaddle)>()
    {
        let mut input_velocity = movement.input_velocity;
        debug_assert!(
            input_velocity == Vec3::ZERO || input_velocity.is_normalized(),
            "Paddle input velocity needs to be normalized"
        );
        if input_velocity.length_squared() < 1e-4 {
            velocity.0 = Vec3::ZERO;
        } else {
            input_velocity *= movement.speed;

            // Restrict vertical movement when colliding with top or bottom boundary
            let relevant_collisions = collisions
                .iter()
                .filter(|e| e.a == entity || e.b == Some(entity));
            let current_position = transform.0.w_axis.xyz();
            for collision in relevant_collisions {
                if collision.info.contact_point.y > current_position.y {
                    input_velocity.y = input_velocity.y.min(0.0);
                } else {
                    input_velocity.y = input_velocity.y.max(0.0);
                }
            }

            velocity.0 = input_velocity;
        }
    }
}

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
