use std::{error::Error, fs};

use glam::{Mat3, Quat, Vec3};
use glow::{HasContext, NativeBuffer, NativeUniformLocation};

use crate::{camera::Camera, objmesh::ObjMesh, scene::Renderer, voxel::Voxel};

pub struct CubeRenderBatch {
    vao: <glow::Context as HasContext>::VertexArray,
    // Contains cube positions contained in this batch
    instance_vbo: NativeBuffer,
    // Number of meshes rendered
    instance_count: i32,
}

impl CubeRenderBatch {
    pub fn new(
        gl: &glow::Context,
        vertex_position_vbo: NativeBuffer,
        vertex_normal_vbo: NativeBuffer,
        cubes: &[Voxel],
    ) -> Result<CubeRenderBatch, Box<dyn Error>> {
        let size = cubes.len();
        debug_assert!(size <= BATCH_SIZE);
        let mut positions_vec: Vec<Vec3> = Vec::with_capacity(cubes.len());
        for cube in cubes {
            positions_vec.push(Vec3::new(cube.position.x, cube.position.y, cube.position.z));
        }
        let positons_bytes: &[u8] = bytemuck::cast_slice(&positions_vec);

        // Setup buffers and vertex attributes
        unsafe {
            // Buffer vertex position data
            let instance_vbo = gl.create_buffer().expect("Cannot create instance vbo");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(instance_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positons_bytes, gl::STATIC_DRAW);

            // Every batch needs its own vertex array object
            let vao = gl
                .create_vertex_array()
                .expect("Cannot create vertex array");

            // Setup position attribute
            gl.bind_vertex_array(Some(vao));
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_position_vbo));
            gl.vertex_attrib_pointer_f32(0, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 0);
            // Setup normal attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_normal_vbo));
            gl.vertex_attrib_pointer_f32(1, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 1);
            // Setup location attribute
            gl.enable_vertex_attrib_array(2);
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(instance_vbo));
            gl.vertex_attrib_pointer_f32(2, 3, gl::FLOAT, false, 0, 0);
            // Update vertex attribute at index 2 on every new instance
            gl.vertex_attrib_divisor(2, 1);

            // Cleanup
            gl.bind_buffer(gl::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);
            Ok(Self {
                instance_vbo,
                vao,
                instance_count: size as i32,
            })
        }
    }

    pub fn render(&self, gl: &glow::Context, vertex_count: i32) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, vertex_count, self.instance_count);
            gl.bind_vertex_array(None);
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.instance_vbo);
            gl.delete_vertex_array(self.vao);
        }
    }
}

// Batch renders cubes
pub struct CubeRenderer {
    // INIT
    program: <glow::Context as HasContext>::Program,
    // vertex vbos will be shared across batches
    vertex_position_vbo: NativeBuffer,
    vertex_normal_vbo: NativeBuffer,

    // Uniform locations
    view_loc: Option<NativeUniformLocation>,
    projection_loc: Option<NativeUniformLocation>,
    light_dir_loc: Option<NativeUniformLocation>,
    color_loc: Option<NativeUniformLocation>,
    mesh: ObjMesh,

    // RUNTIME
    batches: Vec<CubeRenderBatch>,
    pub color: Vec3,
}

const BATCH_SIZE: usize = 1024 * 1024;

impl CubeRenderer {
    pub fn new(gl: &glow::Context) -> Result<CubeRenderer, Box<dyn Error>> {
        let color = Vec3::new(1.0, 0.0, 0.0);
        // FIX: Will have to copy assets in build step for portability
        let vert_src = fs::read_to_string("assets/shaders/cube.vert")?;
        let frag_src = fs::read_to_string("assets/shaders/cube-outline.frag")?;
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
            // Compile shaders & load program
            let program = gl.create_program().expect("Cannot create program");
            for (kind, source, handle) in &mut shaders {
                let shader = gl.create_shader(*kind).expect("Cannot create shader");
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

            // Buffer common vertex data
            // Positions
            let positions_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
            // Normals
            let normals_vbo = gl
                .create_buffer()
                .expect("Cannot create buffer for normals");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normal_bytes, gl::STATIC_DRAW);
            gl.bind_buffer(gl::ARRAY_BUFFER, None);

            // Setup uniforms
            let view_loc = gl.get_uniform_location(program, "uView");
            let projection_loc = gl.get_uniform_location(program, "uProjection");
            let light_dir_loc = gl.get_uniform_location(program, "uLightDir");
            let color_loc = gl.get_uniform_location(program, "uColor");
            Ok(Self {
                color,
                vertex_position_vbo: positions_vbo,
                vertex_normal_vbo: normals_vbo,
                batches: vec![],
                program,
                mesh,
                view_loc,
                projection_loc,
                light_dir_loc,
                color_loc,
            })
        }
    }

    // Needs to be called everytime a cube is transformed, added or removed
    pub fn update_batches(
        &mut self,
        gl: &glow::Context,
        cubes: &[Voxel],
    ) -> Result<(), Box<dyn Error>> {
        // Cleanup: Remove existing batches
        // This ensures that buffers and other gpu resources are released
        for batch in &self.batches {
            batch.destroy(gl);
        }

        // Initialize new buffers
        let batch_count = (cubes.len() as f32 / (BATCH_SIZE as f32)).ceil() as usize;
        self.batches = Vec::with_capacity(batch_count);
        for i in 0..batch_count {
            let start = i * BATCH_SIZE;
            let end = cubes.len().min((i + 1) * BATCH_SIZE);
            let batch = CubeRenderBatch::new(
                gl,
                self.vertex_position_vbo,
                self.vertex_normal_vbo,
                &cubes[start..end],
            )?;
            self.batches.push(batch);
        }
        println!(
            "Updated batches: Now running {} batches for {} cubes",
            batch_count,
            cubes.len()
        );
        Ok(())
    }
}

impl Renderer for CubeRenderer {
    fn render(&self, gl: &glow::Context, cam: &Camera) {
        let view = cam.get_view_matrix();
        let projection = cam.get_projection_matrix();

        // Calculate light direction and transform to camera view space
        let world_space_light_dir = Quat::from_rotation_x(20.0) * Vec3::Y;
        let view_space_light_dir =
            Mat3::from_mat4(cam.get_view_matrix()).mul_vec3(world_space_light_dir);

        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(
                self.view_loc.as_ref(),
                false,
                view.to_cols_array().as_ref(),
            );
            gl.uniform_matrix_4_f32_slice(
                self.projection_loc.as_ref(),
                false,
                projection.to_cols_array().as_ref(),
            );
            gl.uniform_3_f32_slice(
                self.light_dir_loc.as_ref(),
                view_space_light_dir.to_array().as_ref(),
            );
            gl.uniform_3_f32_slice(self.color_loc.as_ref(), self.color.to_array().as_ref());
            // NOTE: /3 because we have 3 coordinates per vertex
            let vertex_count = self.mesh.get_vertex_buffers().position_buffer.len() as i32 / 3;
            for batch in &self.batches {
                batch.render(gl, vertex_count);
            }
        }
    }

    fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_buffer(self.vertex_position_vbo);
            gl.delete_buffer(self.vertex_normal_vbo);
        }
    }
}
