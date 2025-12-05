use std::{error::Error, rc::Rc};

use glam::{Mat4, Vec3};
use glow::HasContext;

use crate::{renderer::shader::Shader, scenes::Renderer};

use super::objmesh::ObjMesh;

pub struct SphereMesh {
    pub position: Vec3,
    // WARN: Not yet fully supported!
    pub radius: f32,
    pub color: Vec3,
    shader: Shader,
    vao: <glow::Context as HasContext>::VertexArray,
    gl: Rc<glow::Context>,
}

impl SphereMesh {
    pub fn new(gl: &Rc<glow::Context>) -> Result<Self, Box<dyn Error>> {
        let shader = Shader::new(
            gl,
            "assets/shaders/sphere.vert",
            "assets/shaders/sphere_rt.frag",
        )?;
        // Load vertex data from mesh
        let mut mesh = ObjMesh::new();
        mesh.load("assets/cube_github.obj")
            .expect("Could not load mesh");
        let vertex_positions = mesh.get_vertex_buffers().position_buffer;
        let vertex_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                vertex_positions.as_ptr() as *const u8,
                vertex_positions.len() * std::mem::size_of::<f32>(),
            )
        };
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
            Ok(Self {
                color: Vec3::new(1.0, 0.0, 0.0),
                gl: Rc::clone(gl),
                position: Vec3::ZERO,
                radius: 0.5,
                shader,
                vao,
            })
        }
    }
}

impl Renderer for SphereMesh {
    fn render(&mut self, cam: &crate::cameras::camera::Camera) {
        self.shader.use_program();
        self.shader
            .set_uniform_mat4("model", &Mat4::from_translation(self.position));
        self.shader.set_uniform_mat4("view", &cam.get_view_matrix());
        self.shader
            .set_uniform_mat4("perspective", &cam.get_projection_matrix());
        self.shader.set_uniform_vec3("camPos", &cam.position);
        self.shader.set_uniform_vec3("sphereCenter", &self.position);
        self.shader.set_uniform_f32("sphereRadius", self.radius);
        self.shader.set_uniform_vec3("sphereColor", &self.color);
        let gl = &self.gl;
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays(gl::TRIANGLES, 0, 36);
            gl.bind_vertex_array(None);
        }
    }
}
