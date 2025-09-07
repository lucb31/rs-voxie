use std::time::Instant;

use glam::{Mat4, Vec3};
use glow::{HasContext, NativeUniformLocation};

use crate::camera::Camera;

// Main source: https://github.com/imgui-rs/imgui-glow-renderer/blob/main/examples/glow_02_triangle.rs
pub struct TriangleRenderer {
    program: <glow::Context as HasContext>::Program,
    vertex_array: <glow::Context as HasContext>::VertexArray,
    start: Instant,
    mvp_loc: Option<NativeUniformLocation>,
}

impl TriangleRenderer {
    pub fn new(gl: &glow::Context) -> TriangleRenderer {
        const SHADER_HEADER: &str = "#version 330";
        const VERTEX_SHADER_SOURCE: &str = r#"
const vec2 verts[3] = vec2[3](
    vec2(-1.0f, -1.0f),
    vec2(-1.0f, 1.0f),
    vec2(1.0f, 1.0f)
);
const vec3 colors[3] = vec3[3](
    vec3(1.0f, 0.0f, 0.0f),
    vec3(0.0f, 1.0f, 0.0f),
    vec3(0.0f, 0.0f, 1.0f)
);
uniform mat4 uMVP;

out vec2 vert;
out vec4 color;

void main() {
    vert = verts[gl_VertexID];
    vec4 vert_vec4 = vec4(vert, 0.0, 1.0);
    color = vec4(colors[gl_VertexID], 1.0f);
    gl_Position = uMVP * vert_vec4;
}
"#;
        const FRAGMENT_SHADER_SOURCE: &str = r#"
in vec2 vert;
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

        unsafe {
            let vertex_array = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

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
        let model = Mat4::from_rotation_z(time);

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
        let mvp = my_cam.get_view_projection_matrix() * model;

        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                self.mvp_loc.as_ref(),
                true,
                mvp.to_cols_array().as_ref(),
            );
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays(glow::TRIANGLES, 0, 3);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }
}
