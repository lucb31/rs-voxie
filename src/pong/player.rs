use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use hecs::World;
use winit::keyboard::KeyCode;

use crate::{
    input::InputState,
    meshes::objmesh::ObjMesh,
    renderer::{Mesh, RenderMeshHandle, ecs_renderer::MESH_CUBE, shader::Shader},
    systems::physics::{Transform, Velocity},
};

struct PongPlayer;
struct PlayerMovement {
    speed: f32,
}

pub fn spawn_player(world: &mut World, position: Vec3) {
    world.spawn((
        PongPlayer,
        Transform(Mat4::from_scale_rotation_translation(
            Vec3::new(0.1, 1.0, 5.0),
            //Vec3::ONE,
            Quat::IDENTITY,
            position,
        )),
        Velocity(Vec3::ZERO),
        RenderMeshHandle(MESH_CUBE),
        PlayerMovement { speed: 5.0 },
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

pub fn mesh_cube(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/cube.vert",
        "assets/shaders/cube-diffuse.frag",
    )?;

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
