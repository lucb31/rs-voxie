use std::{error::Error, fs};

use glam::{Mat3, Mat4, Quat, Vec3, Vec4};
use glow::{Buffer, HasContext, NativeUniformLocation};

use crate::{camera::Camera, objmesh::ObjMesh, scene::Mesh};

pub struct CubeMesh {
    // Transform
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,

    // Color
    pub color: Vec3,
}

pub struct CubeRenderBatch {
    vertex_array: <glow::Context as HasContext>::VertexArray,
}

// Batch renders cubes
pub struct CubeRenderer {
    // INIT
    vertex_array: <glow::Context as HasContext>::VertexArray,
    program: <glow::Context as HasContext>::Program,
    vp_loc: Option<NativeUniformLocation>,
    mv_inverse_transpose_loc: Option<NativeUniformLocation>,
    light_dir_loc: Option<NativeUniformLocation>,
    color_loc: Option<NativeUniformLocation>,
    mesh: ObjMesh,
    // TODO: Batch variable?
    ubo: Buffer,

    // SETUP
    // How many cubes will be rendered per draw call
    batch_size: u32,

    // RUNTIME
    batches: Vec<CubeRenderBatch>,
}

const BATCH_SIZE: u32 = 256;

impl CubeRenderer {
    pub fn new(gl: &glow::Context) -> Result<CubeRenderer, Box<dyn Error>> {
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

            // Setup uniform instance buffer
            let ubo = gl.create_buffer().expect("Cannot create uniform buffer");
            gl.bind_buffer(gl::UNIFORM_BUFFER, Some(ubo));
            gl.buffer_data_size(
                gl::UNIFORM_BUFFER,
                BATCH_SIZE as i32 * std::mem::size_of::<Mat4>() as i32,
                gl::STATIC_DRAW,
            );
            let block_index = gl
                .get_uniform_block_index(program, "InstanceData")
                .expect("Block index not found");
            gl.uniform_block_binding(program, block_index, 0);
            gl.bind_buffer_base(gl::UNIFORM_BUFFER, 0, Some(ubo));

            // Setup regular uniforms
            let vp_loc = gl.get_uniform_location(program, "uViewProjection");
            let mv_inverse_transpose_loc = gl.get_uniform_location(program, "uMvInverseTranspose");
            let light_dir_loc = gl.get_uniform_location(program, "uLightDir");
            let color_loc = gl.get_uniform_location(program, "uColor");
            Ok(Self {
                vertex_array,
                ubo,
                batches: vec![],
                // WARNING: Currently hard-coded into shader. Cannot just change
                batch_size: BATCH_SIZE,
                program,
                mesh,
                vp_loc,
                mv_inverse_transpose_loc,
                light_dir_loc,
                color_loc,
            })
        }
    }

    pub fn update_batches(
        &mut self,
        gl: &glow::Context,
        cubes: &Vec<CubeMesh>,
    ) -> Result<(), Box<dyn Error>> {
        if cubes.len() >= BATCH_SIZE as usize {
            panic!("Missing implementation: Batching. Cannot render that many cubes. ")
        }
        // Needs to be called everytime a cube is transformed, added or removed
        // FIX: V1 ignore batch size, all in one batch
        let batch_count = 1;
        let current_batch_size = cubes.len();
        //self.batches = Vec::with_capacity(batch_count);
        //let batch = CubeRendererBatch::new

        // Loop cubes, get transofmr, buffer transform
        let mut model_matrices: Vec<Mat4> = Vec::with_capacity(current_batch_size);
        for cube in cubes {
            let model = cube.get_transform();
            println!("position: {:?}", cube.position);
            println!("Transform: {:?}", model);
            model_matrices.push(model);
        }
        let model_bytes: &[u8] = bytemuck::cast_slice(&model_matrices);
        println!("{}: {:?}", model_bytes.len(), model_bytes);
        unsafe {
            gl.bind_buffer(gl::UNIFORM_BUFFER, Some(self.ubo));
            gl.buffer_sub_data_u8_slice(gl::UNIFORM_BUFFER, 0, model_bytes);
        }

        Ok(())
    }

    pub fn render(&self, gl: &glow::Context, cam: &Camera) {
        let vp = cam.get_view_projection_matrix();
        // TODO: Need to calc in vert shader because model matrix changes
        // Maybe there's a smarter way to cache
        let mv_inverse = Mat3::from_mat4(cam.get_view_matrix() * Mat4::IDENTITY)
            .inverse()
            .transpose();

        // TODO: How will we distinguish colors?
        let color = Vec3::new(1.0, 0.0, 0.0);
        // Calculate light direction and transform to camera view space
        let world_space_light_dir = Quat::from_rotation_x(20.0) * Vec3::Y;
        let view_space_light_dir =
            Mat3::from_mat4(cam.get_view_matrix()).mul_vec3(world_space_light_dir);

        unsafe {
            gl.use_program(Some(self.program));
            gl.uniform_matrix_4_f32_slice(self.vp_loc.as_ref(), false, vp.to_cols_array().as_ref());
            gl.uniform_matrix_3_f32_slice(
                self.mv_inverse_transpose_loc.as_ref(),
                false,
                mv_inverse.to_cols_array().as_ref(),
            );
            gl.uniform_3_f32_slice(
                self.light_dir_loc.as_ref(),
                view_space_light_dir.to_array().as_ref(),
            );
            gl.uniform_3_f32_slice(self.color_loc.as_ref(), color.to_array().as_ref());
            gl.bind_vertex_array(Some(self.vertex_array));
            gl.draw_arrays_instanced(
                glow::TRIANGLES,
                0,
                self.mesh.get_vertex_buffers().position_buffer.len() as i32,
                BATCH_SIZE as i32,
            );
        }
    }

    pub fn destroy(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_program(self.program);
            gl.delete_vertex_array(self.vertex_array);
        }
    }
}

impl CubeMesh {
    pub fn new() -> Result<CubeMesh, Box<dyn Error>> {
        let position = Vec3::ZERO;
        let rotation = Quat::IDENTITY;
        let scale = Vec3::ONE;
        let color = Vec3::ONE;
        Ok(Self {
            color,
            position,
            rotation,
            scale,
        })
    }

    fn get_transform(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }
}

impl Mesh for CubeMesh {
    fn render(&self, gl: &glow::Context, cam: &Camera) {}
    fn destroy(&self, gl: &glow::Context) {}
    fn tick(&mut self, dt: f32) {
        let speed = 0.0;
        // Make the model rotate
        if self.scale == Vec3::ONE {
            self.rotation *= Quat::from_rotation_y(speed * dt)
        }
    }
}
