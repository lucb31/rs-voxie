use glow::HasContext;
use std::{error::Error, path::Path, rc::Rc};

use glam::{Mat3, Mat4, Quat, Vec3};

use crate::{
    cameras::camera::Camera,
    renderer::{shader::Shader, texture::Texture},
};

use super::objmesh::ObjMesh;

pub struct CubeMesh {
    gl: Rc<glow::Context>,
    pub position: Vec3,
    pub rotation: Quat,

    diff_texture: Texture,
    normal_texture: Texture,
    vao: <glow::Context as HasContext>::VertexArray,
    vertex_count: usize,
    shader: Shader,
}

impl CubeMesh {
    pub fn new(gl: &Rc<glow::Context>) -> Result<CubeMesh, Box<dyn Error>> {
        let mut shader = Shader::new(
            gl.clone(),
            "assets/shaders/cube-diffuse-normal.vert",
            "assets/shaders/cube-diffuse-normal.frag",
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
        let (tangents, bitangents) = mesh.get_tangent_space_buffers();
        let tangent_bytes: &[u8] = bytemuck::cast_slice(&tangents);
        let bitangent_bytes: &[u8] = bytemuck::cast_slice(&bitangents);

        unsafe {
            // Buffer position data
            let positions_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positions_bytes, gl::STATIC_DRAW);
            // Buffer normal data
            let normals_vbo = gl
                .create_buffer()
                .expect("Cannot create buffer for normals");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normals_bytes, gl::STATIC_DRAW);
            // Buffer texture coordinate data
            let tex_coords_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(tex_coords_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, tex_coords_bytes, gl::STATIC_DRAW);
            // Buffer tangent vector coordinate data
            let tangent_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(tangent_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, tangent_bytes, gl::STATIC_DRAW);
            // Buffer bitangent vector coordinate data
            let bitangent_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(bitangent_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, bitangent_bytes, gl::STATIC_DRAW);

            gl.bind_buffer(gl::ARRAY_BUFFER, None);

            // Setup attribute bindings
            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            // Setup position attribute
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
            gl.vertex_attrib_pointer_f32(0, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 0);
            // Setup normal attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
            gl.vertex_attrib_pointer_f32(1, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 1);
            // Setup tex_coords attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(tex_coords_vbo));
            gl.vertex_attrib_pointer_f32(2, 2, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 2);
            // Setup tangent attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(tangent_vbo));
            gl.vertex_attrib_pointer_f32(3, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 3);
            // Setup bitangent attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(bitangent_vbo));
            gl.vertex_attrib_pointer_f32(4, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 4);

            // Load textures
            let diff_texture = Texture::new(gl, Path::new("assets/textures/dirt.png"))
                .expect("Could not load diff texture");
            shader.use_program();
            shader.set_uniform_i32("diffuseMap", 0);
            let normal_texture = Texture::new(gl, Path::new("assets/textures/dirt_n.png"))
                .expect("Could not load normal texture");
            shader.set_uniform_i32("normalMap", 1);

            // Cleanup
            gl.bind_buffer(gl::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            Ok(Self {
                gl: gl.clone(),
                position: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                shader,
                vao,
                diff_texture,
                normal_texture,
                vertex_count,
            })
        }
    }

    pub fn render(&mut self, gl: &glow::Context, cam: &Camera) {
        let view_pos = cam.position;
        let view = cam.get_view_matrix();
        let projection = cam.get_projection_matrix();
        let model = Mat4::from_rotation_translation(self.rotation, self.position);
        let model_inverse_transpose = Mat3::from_mat4(model.inverse().transpose());

        // Calculate light direction and transform to camera view space
        let world_space_light_dir = Vec3::Y;
        let view_space_light_dir =
            Mat3::from_mat4(cam.get_view_matrix()).mul_vec3(world_space_light_dir);

        self.shader.use_program();
        unsafe {
            self.gl.active_texture(gl::TEXTURE0);
            self.diff_texture.bind();
            self.gl.active_texture(gl::TEXTURE1);
            self.normal_texture.bind();
        }
        self.shader.set_uniform_mat4("uView", &view);
        self.shader.set_uniform_mat4("uProjection", &projection);
        self.shader.set_uniform_mat4("uModel", &model);
        self.shader
            .set_uniform_mat3("uModelInverseTranspose", &model_inverse_transpose);
        self.shader
            .set_uniform_vec3("uLightDir", &view_space_light_dir);
        self.shader.set_uniform_vec3("uViewPos", &view_pos);

        unsafe {
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl
                .draw_arrays(glow::TRIANGLES, 0, self.vertex_count as i32);
            self.gl.bind_vertex_array(None);
        }
    }
}
