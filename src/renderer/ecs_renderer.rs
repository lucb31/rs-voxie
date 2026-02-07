use std::{collections::HashMap, error::Error, rc::Rc};

use glam::{Mat3, Vec3};
use glow::HasContext;
use hecs::World;
use log::{debug, error};

use crate::{
    cameras::{camera::Camera, component::CameraComponent},
    systems::{physics::Transform, skybox::quad_mesh},
};

use super::{
    meshes::{mesh_cube, player_mesh, projectile_mesh, projectile2d_mesh, squid::squid_mesh},
    shader::Shader,
};

type MeshHandle = usize;

pub const MESH_PROJECTILE: MeshHandle = 0;
pub const MESH_PLAYER: MeshHandle = 1;
pub const MESH_QUAD: MeshHandle = 2;
pub const MESH_CUBE: MeshHandle = 3;
pub const MESH_PROJECTILE_2D: MeshHandle = 4;
pub const MESH_SQUID: MeshHandle = 5;

pub struct Mesh {
    shader: Shader,
    vao: <glow::Context as HasContext>::VertexArray,
    vertex_count: i32,
    // Interims fix / tag to distinguish between draw_element and draw_arrays mesh implementations
    use_index: bool,
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
            use_index: false,
        }
    }

    pub fn enable_indexed_draw(&mut self) {
        self.use_index = true;
    }
}

/// ECS-based renderer
pub struct ECSRenderer {
    gl: Rc<glow::Context>,
    meshes: HashMap<MeshHandle, Mesh>,
}

#[derive(Clone)]
pub struct RenderMeshHandle(pub usize);
#[derive(Clone)]
pub struct RenderColor(pub Vec3);

impl ECSRenderer {
    pub fn new(gl: &Rc<glow::Context>) -> Result<ECSRenderer, Box<dyn Error>> {
        // Prepare rendering
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let mut instance = Self {
            gl: Rc::clone(gl),
            meshes: HashMap::new(),
        };
        instance.add_mesh(MESH_PROJECTILE, projectile_mesh(gl)?);
        instance.add_mesh(MESH_PLAYER, player_mesh(gl)?);
        instance.add_mesh(MESH_QUAD, quad_mesh(gl)?);
        instance.add_mesh(MESH_CUBE, mesh_cube(gl)?);
        instance.add_mesh(MESH_PROJECTILE_2D, projectile2d_mesh(gl)?);
        instance.add_mesh(MESH_SQUID, squid_mesh(gl)?);

        Ok(instance)
    }

    pub fn add_mesh(&mut self, handle: MeshHandle, mesh: Mesh) -> MeshHandle {
        self.meshes.insert(handle, mesh);
        handle
    }

    pub fn get_mesh(&mut self, handle: MeshHandle) -> Option<&mut Mesh> {
        self.meshes.get_mut(&handle)
    }

    /// Renders world from view of main camera. Will query for camera within world first
    pub fn render(&mut self, world: &World) {
        match query_main_camera(world) {
            Some(cam) => {
                self.render_camera(world, &cam);
            }
            None => {
                error!("Cannot render scene: No camera found");
            }
        };
    }

    pub fn render_camera(&mut self, world: &World, cam: &Camera) {
        // TODO: Instanced draws for same handle
        for (entity, (transform, handle)) in world.query::<(&Transform, &RenderMeshHandle)>().iter()
        {
            debug!("Rendering {entity:?} at {:?}", transform.0);
            let mesh = self
                .get_mesh(handle.0)
                .expect("Invalid mesh handle assigned");
            let use_index = mesh.use_index;
            mesh.shader.use_program();
            mesh.shader.set_uniform_mat4("uModel", &transform.0);
            // TODO: Should not do this at render time. Expensive
            let model_iv_loc = mesh.shader.get_uniform_location("uModelIV");
            if model_iv_loc.is_some() {
                // Only calculate IV if shader requires it
                let model_inverse_transpose = Mat3::from_mat4(transform.0.inverse().transpose());
                mesh.shader
                    .set_uniform_mat3("uModelIV", &model_inverse_transpose);
            }
            mesh.shader
                .set_uniform_mat4("uView", &cam.get_view_matrix());
            mesh.shader
                .set_uniform_mat4("uProjection", &cam.get_projection_matrix());
            if let Ok(color) = world.get::<&RenderColor>(entity) {
                mesh.shader.set_uniform_vec3("uColor", &color.0);
            }

            let vao = mesh.vao;
            let count = mesh.vertex_count;
            let gl = &self.gl;
            unsafe {
                gl.bind_vertex_array(Some(vao));
                if use_index {
                    gl.draw_elements(glow::TRIANGLES, count, gl::UNSIGNED_INT, 0);
                } else {
                    gl.draw_arrays(gl::TRIANGLES, 0, count);
                }
                gl.bind_vertex_array(None);
            }
        }
    }
}

fn query_main_camera(world: &World) -> Option<Camera> {
    let mut query = world.query::<(&CameraComponent, &Transform)>();
    let (_entity, (cam_component, transform)) = query.iter().next()?;
    let mut cam = Camera::new();
    let (_scale, rot, trans) = transform.0.to_scale_rotation_translation();
    cam.position = trans;
    cam.set_rotation(rot);
    cam.set_projection(cam_component.projection);
    Some(cam)
}
