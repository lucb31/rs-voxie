use std::{error::Error, rc::Rc};

use glam::Mat3;
use glow::HasContext;
use hecs::World;
use log::debug;

use crate::{
    cameras::camera::Camera, ecs::Transform, meshes::objmesh::ObjMesh, player::player_mesh,
};

use super::shader::Shader;

pub const MESH_PROJECTILE: usize = 0;
pub const MESH_PLAYER: usize = 1;

pub struct Mesh {
    shader: Shader,
    vao: <glow::Context as HasContext>::VertexArray,
    vertex_count: i32,
}
impl Mesh {
    pub fn new(
        shader: Shader,
        vao: <glow::Context as HasContext>::VertexArray,
        vertex_count: i32,
    ) -> Mesh {
        Self {
            shader,
            vao,
            vertex_count,
        }
    }
}

/// ECS-based renderer
pub struct ECSRenderer {
    gl: Rc<glow::Context>,
    meshes: Vec<Mesh>,
}

pub struct RenderMeshHandle(pub usize);

impl ECSRenderer {
    pub fn new(gl: &Rc<glow::Context>) -> Result<ECSRenderer, Box<dyn Error>> {
        let meshes = vec![projectile_mesh(gl)?, player_mesh(gl)?];
        Ok(Self {
            gl: Rc::clone(gl),
            meshes,
        })
    }

    pub fn add_mesh(&mut self, mesh: Mesh) -> usize {
        let idx = self.meshes.len();
        self.meshes.push(mesh);
        idx
    }

    pub fn get_mesh(&mut self, handle: usize) -> Option<&mut Mesh> {
        self.meshes.get_mut(handle)
    }

    pub fn render(&mut self, world: &mut World, cam: &Camera) {
        // TODO: Instanced draws for same handle
        for (_entity, (transform, handle)) in world.query_mut::<(&Transform, &RenderMeshHandle)>() {
            debug!("Rendering {_entity:?} at {:?}", transform.0);
            let mesh = self
                .get_mesh(handle.0)
                .expect("Invalid mesh handle assigned");
            mesh.shader.use_program();
            mesh.shader.set_uniform_mat4("uModel", &transform.0);
            let model_iv_loc = mesh.shader.get_uniform_location("uModelIV");
            if model_iv_loc.is_some() {
                let model_inverse_transpose = Mat3::from_mat4(transform.0.inverse().transpose());
                mesh.shader
                    .set_uniform_mat3("uModelIV", &model_inverse_transpose);
            }
            mesh.shader
                .set_uniform_mat4("uView", &cam.get_view_matrix());
            mesh.shader
                .set_uniform_mat4("uProjection", &cam.get_projection_matrix());
            let vao = mesh.vao;
            let count = mesh.vertex_count;
            let gl = &self.gl;
            unsafe {
                gl.bind_vertex_array(Some(vao));
                gl.draw_arrays(gl::TRIANGLES, 0, count);
                gl.bind_vertex_array(None);
            }
        }
    }
}

fn projectile_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/cube.vert",
        "assets/shaders/projectile.frag",
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
        gl.bind_buffer(gl::ARRAY_BUFFER, None);
        Ok(Mesh {
            shader,
            vao,
            vertex_count: vertex_positions.len() as i32,
        })
    }
}
