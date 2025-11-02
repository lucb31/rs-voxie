use std::{collections::HashMap, error::Error, fs, rc::Rc};

use glam::{Mat3, Mat4, Vec3};
use glow::{HasContext, NativeUniformLocation};

pub struct Shader {
    gl: Rc<glow::Context>,
    program: <glow::Context as HasContext>::Program,
    uniforms: HashMap<String, Option<NativeUniformLocation>>,
}

impl Shader {
    pub fn new(
        gl: Rc<glow::Context>,
        vert_path: &str,
        frag_path: &str,
    ) -> Result<Shader, Box<dyn Error>> {
        let vert_src = fs::read_to_string(vert_path)?;
        let frag_src = fs::read_to_string(frag_path)?;
        let mut shaders = [
            (glow::VERTEX_SHADER, vert_src, None),
            (glow::FRAGMENT_SHADER, frag_src, None),
        ];
        unsafe {
            // Compile shaders & load program
            let program = gl.create_program()?;
            for (kind, source, handle) in &mut shaders {
                let shader = gl.create_shader(*kind)?;
                gl.shader_source(shader, source);
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
            let instance = Self {
                gl,
                program,
                uniforms: HashMap::new(),
            };
            instance.check_gl_errors();
            Ok(instance)
        }
    }

    pub fn use_program(&self) {
        unsafe {
            self.gl.use_program(Some(self.program));
        }
    }

    pub fn check_gl_errors(&self) {
        unsafe {
            let mut error = self.gl.get_error();
            while error != glow::NO_ERROR {
                println!("OpenGL Error: 0x{error:X}");
                error = self.gl.get_error();
            }
        }
    }

    fn get_uniform_location(&mut self, name: &str) -> Option<NativeUniformLocation> {
        if let Some(loc) = self.uniforms.get(name) {
            return *loc;
        }
        unsafe {
            let loc = self.gl.get_uniform_location(self.program, name);
            if loc.is_none() {
                println!("ERR: Trying to set unknown uniform {name}");
            }
            self.uniforms.insert(name.to_string(), loc);
            loc
        }
    }

    pub fn set_uniform_i32(&mut self, name: &str, value: i32) {
        let loc = self.get_uniform_location(name);
        unsafe {
            self.gl.uniform_1_i32(loc.as_ref(), value);
        }
    }

    pub fn set_uniform_mat3(&mut self, name: &str, value: &Mat3) {
        let loc = self.get_uniform_location(name);
        unsafe {
            self.gl
                .uniform_matrix_3_f32_slice(loc.as_ref(), false, value.to_cols_array().as_ref());
        }
    }

    pub fn set_uniform_mat4(&mut self, name: &str, value: &Mat4) {
        let loc = self.get_uniform_location(name);
        unsafe {
            self.gl
                .uniform_matrix_4_f32_slice(loc.as_ref(), false, value.to_cols_array().as_ref());
        }
    }

    pub fn set_uniform_f32(&mut self, name: &str, value: f32) {
        let loc = self.get_uniform_location(name);
        unsafe {
            self.gl.uniform_1_f32(loc.as_ref(), value);
        }
    }

    pub fn set_uniform_vec3(&mut self, name: &str, value: &Vec3) {
        let loc = self.get_uniform_location(name);
        unsafe {
            self.gl
                .uniform_3_f32_slice(loc.as_ref(), value.to_array().as_ref());
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}
