use std::{error::Error, fs};

use glam::{Mat3, Mat4, Quat, Vec3};
use glow::{HasContext, NativeUniformLocation};

use crate::{camera::Camera, objmesh::ObjMesh, scene::Mesh};

pub struct CubeMesh {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    mvp_loc: Option<NativeUniformLocation>,
    mv_inverse_transpose_loc: Option<NativeUniformLocation>,
    light_dir_loc: Option<NativeUniformLocation>,
    color_loc: Option<NativeUniformLocation>,
    mesh: ObjMesh,

    // Transform
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    // Color
    pub color: Vec3,
}

impl CubeMesh {
    pub fn new(gl: &glow::Context) -> Result<CubeMesh, Box<dyn Error>> {
        // FIX: Will have to copy assets in build step for portability
        let vert_src = fs::read_to_string("assets/shaders/cube.vert")?;
        let frag_src = fs::read_to_string("assets/shaders/cube.frag")?;
        let mut shaders = [
            (glow::VERTEX_SHADER, vert_src, None),
            (glow::FRAGMENT_SHADER, frag_src, None),
        ];

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
        let vertex_normals = mesh.get_vertex_buffers().normal_buffer;
        let normal_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                vertex_normals.as_ptr() as *const u8,
                vertex_normals.len() * std::mem::size_of::<f32>(),
            )
        };
        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            for (kind, source, handle) in &mut shaders {
                let shader = gl.create_shader(*kind).expect("Cannot create shader");
                gl.shader_source(shader, &source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("{}", gl.get_shader_info_log(shader));
                }
                gl.attach_shader(program, shader);
                *handle = Some(shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                panic!("{}", gl.get_program_info_log(program));
            }

            for &(_, _, shader) in &shaders {
                gl.detach_shader(program, shader.unwrap());
                gl.delete_shader(shader.unwrap());
            }

            // Setup vertex array and buffer
            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            let vertex_buffer = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_vertex_array(Some(vertex_array));
            // Bind vertex data
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
            // Setup position attribute
            gl.vertex_attrib_pointer_f32(0, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vertex_array, 0);

            // Setup normal buffer
            let normal_buffer = gl
                .create_buffer()
                .expect("Cannot create buffer for normals");
            // Bind normal data
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(normal_buffer));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normal_bytes, gl::STATIC_DRAW);
            // Setup normal attribute
            gl.vertex_attrib_pointer_f32(1, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vertex_array, 1);

            let mvp_loc = gl.get_uniform_location(program, "uMVP");
            let mv_inverse_transpose_loc = gl.get_uniform_location(program, "uMvInverseTranspose");
            let light_dir_loc = gl.get_uniform_location(program, "uLightDir");
            let color_loc = gl.get_uniform_location(program, "uColor");

            let position = Vec3::ZERO;
            let rotation = Quat::from_rotation_y(0.0);
            let scale = Vec3::ONE;
            let color = Vec3::new(1.0, 1.0, 1.0);
            Ok(Self {
                color,
                color_loc,
                mvp_loc,
                program,
                vertex_array,
                mv_inverse_transpose_loc,
                light_dir_loc,
                mesh,
                position,
                rotation,
                scale,
            })
        }
    }

    fn get_transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

impl Mesh for CubeMesh {
    fn render(&self, gl: &glow::Context, cam: &Camera) {
        let model = self.get_transform();
        let mvp = cam.get_view_projection_matrix() * model;
        let mv_inverse = Mat3::from_mat4(cam.get_view_matrix() * model)
            .inverse()
            .transpose();

        // Calculate light direction and transform to camera view space
        let world_space_light_dir = Quat::from_rotation_x(20.0) * Vec3::Y;
        let view_space_light_dir =
            Mat3::from_mat4(cam.get_view_matrix()).mul_vec3(world_space_light_dir);

        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                self.mvp_loc.as_ref(),
                false,
                mvp.to_cols_array().as_ref(),
            );
            gl.uniform_matrix_3_f32_slice(
                self.mv_inverse_transpose_loc.as_ref(),
                false,
                mv_inverse.to_cols_array().as_ref(),
            );
            gl.uniform_3_f32_slice(
                self.light_dir_loc.as_ref(),
                view_space_light_dir.to_array().as_ref(),
            );
            gl.uniform_3_f32_slice(self.color_loc.as_ref(), self.color.to_array().as_ref());
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(
                glow::TRIANGLES,
                0,
                self.mesh.get_vertex_buffers().position_buffer.len() as i32,
            );
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }

    fn tick(&mut self, dt: f32) {
        let speed = 0.0;
        // Make the model rotate
        if self.scale == Vec3::ONE {
            self.rotation *= Quat::from_rotation_y(speed * dt)
        }
    }
}
