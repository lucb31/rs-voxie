use std::{
    cell::RefCell,
    error::Error,
    fs,
    rc::Rc,
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread,
    time::Instant,
};

use glam::{Mat3, Quat, Vec3};
use glow::{HasContext, NativeBuffer, NativeUniformLocation};

use crate::{
    cameras::camera::Camera,
    meshes::objmesh::ObjMesh,
    octree::IAabb,
    scene::Renderer,
    voxel::{CHUNK_SIZE, VoxelChunk, VoxelKind},
    world::VoxelWorld,
};

pub struct CubeRenderBatch {
    vao: <glow::Context as HasContext>::VertexArray,
    gl: Rc<glow::Context>,
    // Contains cube positions contained in this batch
    instance_vbo: NativeBuffer,
    // Number of meshes rendered
    pub instance_count: i32,
}

impl CubeRenderBatch {
    pub fn new(
        gl: Rc<glow::Context>,
        vertex_position_vbo: NativeBuffer,
        vertex_normal_vbo: NativeBuffer,
        positions_vec: &[Vec3],
    ) -> Result<CubeRenderBatch, Box<dyn Error>> {
        let size = positions_vec.len();
        debug_assert!(size <= BATCH_SIZE);
        let positons_bytes: &[u8] = bytemuck::cast_slice(positions_vec);

        // Setup buffers and vertex attributes
        unsafe {
            let start_buffering = Instant::now();
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

            println!(
                "GPU buffering of {} instances took {}s",
                positions_vec.len(),
                start_buffering.elapsed().as_secs_f32()
            );
            Ok(Self {
                gl,
                instance_vbo,
                vao,
                instance_count: positions_vec.len() as i32,
            })
        }
    }

    pub fn render(&mut self, gl: &glow::Context, vertex_count: i32) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, vertex_count, self.instance_count);
            gl.bind_vertex_array(None);
        }
    }
}

impl Drop for CubeRenderBatch {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.instance_vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

// Batch renders cubes
pub struct CubeRenderer {
    // INIT
    gl: Rc<glow::Context>,
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
    world: Rc<RefCell<VoxelWorld>>,
    pub color: Vec3,

    // Need to update batches
    pub is_dirty: bool,
    batch_thread_receiver: Option<Receiver<Vec<Vec<Vec3>>>>,
}

const BATCH_SIZE: usize = 1024 * 1024;

impl CubeRenderer {
    pub fn new(
        gl: Rc<glow::Context>,
        world: Rc<RefCell<VoxelWorld>>,
    ) -> Result<CubeRenderer, Box<dyn Error>> {
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
                batch_thread_receiver: None,
                is_dirty: true,
                gl,
                world,
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

    fn update(&mut self, camera_fov: &IAabb) -> Result<(), Box<dyn Error>> {
        if let Some(batch_channel) = &self.batch_thread_receiver {
            // Thread already started. Check status
            match batch_channel.try_recv() {
                Ok(position_vecs) => {
                    // println!("Finally done. Let's assemble batches and swap");
                    let mut new_batches = Vec::with_capacity(position_vecs.len());
                    for pos_vec in &position_vecs {
                        let batch = CubeRenderBatch::new(
                            self.gl.clone(),
                            self.vertex_position_vbo,
                            self.vertex_normal_vbo,
                            pos_vec,
                        )?;
                        new_batches.push(batch);
                    }
                    // Swap batches: Remove existing batches
                    // Existing buffers will automatically be removed & ensure ensure that buffers and other gpu resources are released
                    // by implementing the drop trait
                    self.batches = new_batches;
                    self.batch_thread_receiver = None;
                    self.is_dirty = false;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // println!("Task still running...");
                }
                Err(err) => {
                    println!("Task sender was dropped unexpectedly.");
                }
            }
        } else {
            // Update requested, but no thread started yet. Need to spin one up
            let (tx, rx) = mpsc::channel();
            self.batch_thread_receiver = Some(rx);
            let chunks = self.world.borrow().query_region_chunks(camera_fov);
            thread::spawn(move || {
                let new_batches = generate_position_vecs(&chunks);
                tx.send(new_batches).unwrap();
            });
        }
        Ok(())
    }

    pub fn get_instance_count(&self) -> i32 {
        let mut count = 0;
        for batch in &self.batches {
            count += batch.instance_count;
        }
        count
    }

    pub fn tick(&mut self, dt: f32, camera_fov: &IAabb) {
        if self.is_dirty {
            println!("filthy cube renderer");
            self.update(camera_fov).expect("Could not update");
        }
    }
}

// Generate a vector of Vec3s for every batch.
// We dont want to assemble batches here directly as that would
// require access to gl context, therefore complicated locking etc.
fn generate_position_vecs(chunks: &[Arc<VoxelChunk>]) -> Vec<Vec<Vec3>> {
    let start_generation = Instant::now();
    let batch_count = ((chunks.len() * CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE) as f32
        / (BATCH_SIZE as f32))
        .ceil() as usize;
    let mut position_vecs = Vec::with_capacity(batch_count);
    let mut position_vec: Vec<Vec3> = Vec::with_capacity(BATCH_SIZE);
    let mut rendered_voxels: usize = 0;
    // NOTE: If we keep track of which batch houses which chunks, we might be able
    // to do smart updates. Maybe not when the FoV updates, but at least when
    // voxel data of a chunk is altered. We'll only need to update that batch
    for chunk in chunks {
        // Check if there's enough space
        let slice = chunk.voxel_slice();
        if position_vec.len() + slice.len() > BATCH_SIZE {
            println!("Cannot fit entire chunk into current batch. Creating new batch");
            // Finish batch
            rendered_voxels += position_vec.len();
            position_vecs.push(position_vec);
            position_vec = Vec::with_capacity(BATCH_SIZE);
        }
        for cube in slice {
            if matches!(cube.kind, VoxelKind::Air) {
                continue;
            }
            position_vec.push(cube.position);
        }
    }
    // Push final batch
    rendered_voxels += position_vec.len();
    position_vecs.push(position_vec);
    println!(
        "Generating {} batches for {} visible voxels took {}ms",
        position_vecs.len(),
        rendered_voxels,
        start_generation.elapsed().as_secs_f32() * 1000.0
    );
    position_vecs
}

impl Renderer for CubeRenderer {
    fn render(&mut self, gl: &glow::Context, cam: &Camera) {
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
            for batch in &mut self.batches {
                batch.render(gl, vertex_count);
            }
        }
    }
}

impl Drop for CubeRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
            self.gl.delete_buffer(self.vertex_position_vbo);
            self.gl.delete_buffer(self.vertex_normal_vbo);
        }
    }
}
