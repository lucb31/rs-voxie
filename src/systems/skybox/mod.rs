use std::{error::Error, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use glow::HasContext;
use hecs::World;

use crate::renderer::{
    Mesh, RenderMeshHandle,
    ecs_renderer::{MESH_QUAD, RenderColor},
    shader::Shader,
};

use super::physics::Transform;

/// Setup world boundary planes planes
pub fn spawn_skybox(world: &mut World) {
    let render_mesh_handle = RenderMeshHandle(MESH_QUAD);
    world.spawn_batch([
        (
            // Bottom
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::splat(1e3),
                Quat::from_rotation_x(-90f32.to_radians()),
                Vec3::ZERO,
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::Y),
        ),
        (
            // Top
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::splat(1e3),
                Quat::from_rotation_x(90f32.to_radians()),
                Vec3::new(0.0, 1e3, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::Y),
        ),
        (
            // Right
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::splat(1e3),
                Quat::from_rotation_y(90f32.to_radians()),
                Vec3::ZERO,
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::X),
        ),
        (
            // Left
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::splat(1e3),
                Quat::from_rotation_y(-90f32.to_radians()),
                Vec3::new(1e3, 0.0, 0.0),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::X),
        ),
        (
            // Front
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::splat(1e3),
                Quat::from_rotation_y(-180f32.to_radians()),
                Vec3::new(0.0, 0.0, 1e3),
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::Z),
        ),
        (
            // Back
            Transform(Mat4::from_scale_rotation_translation(
                Vec3::splat(1e3),
                Quat::from_rotation_z(90f32.to_radians()),
                Vec3::ZERO,
            )),
            render_mesh_handle.clone(),
            RenderColor(Vec3::Z),
        ),
    ]);
}
pub fn quad_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/quad.vert",
        "assets/shaders/checkerboard-3d.frag",
    )?;
    quad_vertex_mesh(gl, shader)
}

pub fn fog_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let mut shader = Shader::new(gl, "assets/shaders/fog.vert", "assets/shaders/fog.frag")?;
    shader.use_program();
    shader.set_uniform_i32("screenTexture", 0);
    shader.set_uniform_i32("depthTexture", 1);
    quad_vertex_mesh(gl, shader)
}

fn quad_vertex_mesh(gl: &Rc<glow::Context>, shader: Shader) -> Result<Mesh, Box<dyn Error>> {
    let vertex_positions: [f32; 2 * 4] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
    let vertex_bytes: &[u8] = bytemuck::cast_slice(&vertex_positions);
    let tex_coordinates: [f32; 2 * 4] = [0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 0.0];
    let tex_coordinate_bytes: &[u8] = bytemuck::cast_slice(&tex_coordinates);

    let indices: [u32; 6] = [1, 0, 2, 2, 0, 3];
    let index_bytes: &[u8] = bytemuck::cast_slice(&indices);
    unsafe {
        // Setup vao
        let vao = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vao));

        // Bind vertex data
        let vertex_buffer = gl.create_buffer().expect("Cannot create vertex buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_buffer));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(
            0,
            2,
            gl::FLOAT,
            false,
            2 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_array_attrib(vao, 0);

        // Bind tex coordinate data
        let tex_buffer = gl.create_buffer().expect("Cannot create vertex buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(tex_buffer));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, tex_coordinate_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(
            1,
            2,
            gl::FLOAT,
            false,
            2 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_array_attrib(vao, 1);

        // Bind index data
        let element_buffer = gl
            .create_buffer()
            .expect("Cannot create buffer for indices");
        gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(element_buffer));
        gl.buffer_data_u8_slice(gl::ELEMENT_ARRAY_BUFFER, index_bytes, gl::STATIC_DRAW);
        gl.bind_vertex_array(None);
        let mut mesh = Mesh::new(shader, vao, 6);
        mesh.enable_indexed_draw();
        Ok(mesh)
    }
}
