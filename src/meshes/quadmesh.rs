use std::{error::Error, fs, rc::Rc};

use glam::{Mat4, Quat, Vec3};
use glow::{HasContext, NativeUniformLocation};

use crate::{camera::Camera, scene::Renderer};

pub struct QuadMesh {
    gl: Rc<glow::Context>,
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    transform_loc: Option<NativeUniformLocation>,

    // Transform
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl QuadMesh {
    pub fn new(gl: Rc<glow::Context>) -> Result<QuadMesh, Box<dyn Error>> {
        // FIX: Will have to copy assets in build step for portability
        let vert_src = fs::read_to_string("assets/shaders/quad.vert")?;
        let frag_src = fs::read_to_string("assets/shaders/checkerboard-3d.frag")?;
        let mut shaders = [
            (glow::VERTEX_SHADER, vert_src, None),
            (glow::FRAGMENT_SHADER, frag_src, None),
        ];

        let vertex_positions: [f32; 2 * 4] = [-1.0, -1.0, -1.0, 1.0, 1.0, 1.0, 1.0, -1.0];
        let vertex_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                vertex_positions.as_ptr() as *const u8,
                vertex_positions.len() * std::mem::size_of::<f32>(),
            )
        };
        let indices: [u32; 6] = [1, 0, 2, 2, 0, 3];
        let index_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                indices.as_ptr() as *const u8,
                indices.len() * std::mem::size_of::<u32>(),
            )
        };
        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            for (kind, source, handle) in &mut shaders {
                let shader = gl.create_shader(*kind).expect("Cannot create shader");
                gl.shader_source(shader, source);
                gl.compile_shader(shader);
                if !gl.get_shader_compile_status(shader) {
                    panic!("Cannot compile shader: {}", gl.get_shader_info_log(shader));
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

            // Setup vertex & index array and buffer
            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            let vertex_buffer = gl.create_buffer().expect("Cannot create vertex buffer");
            let element_buffer = gl
                .create_buffer()
                .expect("Cannot create buffer for indices");
            gl.bind_vertex_array(Some(vertex_array));
            // Bind vertex data
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
            gl.enable_vertex_array_attrib(vertex_array, 0);

            // Bind index data
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(element_buffer));
            gl.buffer_data_u8_slice(gl::ELEMENT_ARRAY_BUFFER, index_bytes, gl::STATIC_DRAW);

            let transform_loc = gl.get_uniform_location(program, "uTransform");

            let position = Vec3::ZERO;
            let rotation = Quat::from_rotation_y(0.0);
            let scale = Vec3::ONE;
            Ok(Self {
                gl,
                transform_loc,
                program,
                vertex_array,
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

impl Drop for QuadMesh {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_vertex_array(self.vertex_array);
        }
    }
}

impl Renderer for QuadMesh {
    fn render(&mut self, gl: &glow::Context, cam: &Camera) {
        let mvp = cam.get_view_projection_matrix() * self.get_transform();
        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                self.transform_loc.as_ref(),
                false,
                mvp.to_cols_array().as_ref(),
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_elements(glow::TRIANGLES, 6, gl::UNSIGNED_INT, 0);
        }
    }
}
