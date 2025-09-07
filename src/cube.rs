use std::time::Instant;

use glam::{Mat4, Vec3};
use glow::{HasContext, NativeUniformLocation};

use crate::camera::Camera;

// Vertex data for a cube (positions and colors)
const VERTICES: &[f32] = &[
    // positions      // colors
    -0.5, -0.5, -0.5, 1.0, 0.0, 0.0, // red
    0.5, -0.5, -0.5, 0.0, 1.0, 0.0, // green
    0.5, 0.5, -0.5, 0.0, 0.0, 1.0, // blue
    -0.5, 0.5, -0.5, 1.0, 1.0, 0.0, // yellow
    -0.5, -0.5, 0.5, 1.0, 0.0, 1.0, // magenta
    0.5, -0.5, 0.5, 0.0, 1.0, 1.0, // cyan
    0.5, 0.5, 0.5, 1.0, 1.0, 1.0, // white
    -0.5, 0.5, 0.5, 0.0, 0.0, 0.0, // black
];

// Indices for the cube's 12 triangles (two per face)
const INDICES: &[u32] = &[
    0, 1, 2, 2, 3, 0, // back face
    4, 5, 6, 6, 7, 4, // front face
    4, 5, 1, 1, 0, 4, // bottom face
    7, 6, 2, 2, 3, 7, // top face
    4, 0, 3, 3, 7, 4, // left face
    5, 1, 2, 2, 6, 5, // right face
];

pub struct CubeRenderer {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    start: Instant,
    mvp_loc: Option<NativeUniformLocation>,
}

impl CubeRenderer {
    pub fn new(gl: &glow::Context) -> CubeRenderer {
        const SHADER_HEADER: &str = "#version 330";
        const VERTEX_SHADER_SOURCE: &str = r#"
uniform mat4 uMVP;
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aColor;

out vec4 color;

void main() {
    color = vec4(aColor, 1.0);
    gl_Position = uMVP * vec4(aPos, 1.0);
}
"#;
        const FRAGMENT_SHADER_SOURCE: &str = r#"
in vec4 color;

out vec4 frag_color;

void main() {
    frag_color = color;
}
"#;

        let mut shaders = [
            (glow::VERTEX_SHADER, VERTEX_SHADER_SOURCE, None),
            (glow::FRAGMENT_SHADER, FRAGMENT_SHADER_SOURCE, None),
        ];

        // Setup vertex data
        let vertex_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                VERTICES.as_ptr() as *const u8,
                VERTICES.len() * std::mem::size_of::<f32>(),
            )
        };
        let index_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                INDICES.as_ptr() as *const u8,
                INDICES.len() * std::mem::size_of::<u32>(),
            )
        };
        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            for (kind, source, handle) in &mut shaders {
                let shader = gl.create_shader(*kind).expect("Cannot create shader");
                gl.shader_source(shader, &format!("{}\n{}", SHADER_HEADER, *source));
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

            // Setup vertex array and buffers
            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");
            let vertex_buffer = gl.create_buffer().expect("Cannot create buffer");
            let element_buffer = gl.create_buffer().expect("Cannot create buffer");

            // Bind vertex data
            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_buffer));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
            // Bind index data
            gl.bind_buffer(gl::ELEMENT_ARRAY_BUFFER, Some(element_buffer));
            gl.buffer_data_u8_slice(gl::ELEMENT_ARRAY_BUFFER, index_bytes, gl::STATIC_DRAW);
            // Setup position attribute
            gl.vertex_attrib_pointer_f32(
                0,
                3,
                gl::FLOAT,
                false,
                6 * std::mem::size_of::<f32>() as i32,
                0,
            );
            gl.enable_vertex_array_attrib(vertex_array, 0);
            // Setup color attribute
            gl.vertex_attrib_pointer_f32(
                1,
                3,
                gl::FLOAT,
                false,
                6 * std::mem::size_of::<f32>() as i32,
                3 * std::mem::size_of::<f32>() as i32,
            );
            gl.enable_vertex_array_attrib(vertex_array, 1);

            let mvp_loc = gl.get_uniform_location(program, "uMVP");

            Self {
                start: Instant::now(),
                mvp_loc,
                program,
                vertex_array,
            }
        }
    }

    pub fn render(&self, gl: &glow::Context) {
        let time = self.start.elapsed().as_secs_f32();
        // Make the model rotate
        let model = Mat4::from_rotation_x(time);

        // TODO: Use one global camera & pass that to render calls
        let mut my_cam = Camera::new();
        // Make the camera "zoom out"
        let camera_speed = 0.0;
        let additional_movement = Vec3 {
            x: 0.0,
            y: 0.0,
            z: time * camera_speed,
        };
        my_cam.translate(additional_movement);
        // let mvp = my_cam.get_view_projection_matrix() * model;
        let mvp = model;

        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                self.mvp_loc.as_ref(),
                false,
                mvp.to_cols_array().as_ref(),
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_elements(glow::TRIANGLES, INDICES.len() as i32, gl::UNSIGNED_INT, 0);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }
}
