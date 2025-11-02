use std::{
    cell::RefCell,
    error::Error,
    path::Path,
    rc::Rc,
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
    thread,
    time::Instant,
};

use glam::{Quat, Vec3};
use glow::{HasContext, NativeBuffer};
use log::{debug, error, trace};

use crate::{
    cameras::camera::Camera,
    meshes::objmesh::ObjMesh,
    octree::IAabb,
    renderer::{shader::Shader, texture::Texture},
    scene::Renderer,
    voxel::{CHUNK_SIZE, VoxelChunk, VoxelKind},
    world::VoxelWorld,
};

pub struct CubeRenderBatch {
    vao: <glow::Context as HasContext>::VertexArray,
    gl: Rc<glow::Context>,
    texture: Texture,
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
        vertex_tex_coords_vbo: NativeBuffer,
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
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(instance_vbo));
            gl.vertex_attrib_pointer_f32(2, 3, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_attrib_array(2);
            // Update vertex attribute at index 2 on every new instance
            gl.vertex_attrib_divisor(2, 1);
            // Setup tex_coords attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_tex_coords_vbo));
            gl.vertex_attrib_pointer_f32(3, 2, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 3);

            // Load texture
            let texture = Texture::new(&gl, Path::new("assets/textures/dirt.png"))
                .expect("Could not load texture");

            // Cleanup
            gl.bind_buffer(gl::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            trace!(
                "GPU buffering of {} instances took {}s",
                positions_vec.len(),
                start_buffering.elapsed().as_secs_f32()
            );
            Ok(Self {
                gl,
                instance_count: positions_vec.len() as i32,
                instance_vbo,
                texture,
                vao,
            })
        }
    }

    pub fn render(&mut self, gl: &glow::Context, vertex_count: usize) {
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            self.texture.bind();
            gl.draw_arrays_instanced(glow::TRIANGLES, 0, vertex_count as i32, self.instance_count);
            self.texture.unbind();
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

// Responsible for orchestration of cube render batches
pub struct CubeRenderer {
    gl: Rc<glow::Context>,
    shader: Shader,
    // vertex vbos will be shared across batches
    vertex_position_vbo: NativeBuffer,
    vertex_normal_vbo: NativeBuffer,
    vertex_tex_coord_vbo: NativeBuffer,
    vertex_count: usize,

    // RUNTIME
    batches: Vec<CubeRenderBatch>,
    world: Rc<RefCell<VoxelWorld>>,
    pub color: Vec3,

    // Need to update batches; will continue to stay true until update task has been finished
    pub is_dirty: bool,
    batch_thread_receiver: Option<Receiver<Vec<Vec<Vec3>>>>,
}

const BATCH_SIZE: usize = 1024 * 1024;

impl CubeRenderer {
    pub fn new(
        gl: Rc<glow::Context>,
        world: Rc<RefCell<VoxelWorld>>,
    ) -> Result<CubeRenderer, Box<dyn Error>> {
        // Setup shader
        let shader = Shader::new(
            gl.clone(),
            "assets/shaders/voxel.vert",
            "assets/shaders/cube-diffuse.frag",
        )?;

        let color = Vec3::new(1.0, 0.0, 0.0);

        // Load vertex data from mesh
        let mut mesh = ObjMesh::new();
        mesh.load("assets/cube.obj").expect("Could not load mesh");
        let vertex_buffers = mesh.get_vertex_buffers();
        // NOTE: /3 because we have 3 coordinates per vertex
        let vertex_count = vertex_buffers.position_buffer.len() / 3;
        let positions_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.position_buffer);
        let normals_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.normal_buffer);
        let tex_coords_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.tex_coord_buffer);
        unsafe {
            // Buffer position data
            let positions_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positions_bytes, gl::STATIC_DRAW);
            // Buffer normal data
            let normals_vbo = gl
                .create_buffer()
                .expect("Cannot create buffer for normals");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normals_bytes, gl::STATIC_DRAW);
            // Buffer texture coordinate data
            let tex_coords_vbo = gl.create_buffer().expect("Cannot create buffer");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(tex_coords_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, tex_coords_bytes, gl::STATIC_DRAW);
            gl.bind_buffer(gl::ARRAY_BUFFER, None);

            Ok(Self {
                batch_thread_receiver: None,
                batches: vec![],
                color,
                gl,
                is_dirty: true,
                shader,
                vertex_count,
                vertex_normal_vbo: normals_vbo,
                vertex_tex_coord_vbo: tex_coords_vbo,
                vertex_position_vbo: positions_vbo,
                world,
            })
        }
    }

    fn update(&mut self, camera_fov: &IAabb) -> Result<(), Box<dyn Error>> {
        if let Some(batch_channel) = &self.batch_thread_receiver {
            // Thread running, check status
            match batch_channel.try_recv() {
                Ok(position_vecs) => {
                    // Assemble batches
                    let mut new_batches = Vec::with_capacity(position_vecs.len());
                    for pos_vec in &position_vecs {
                        let batch = CubeRenderBatch::new(
                            self.gl.clone(),
                            self.vertex_position_vbo,
                            self.vertex_normal_vbo,
                            self.vertex_tex_coord_vbo,
                            pos_vec,
                        )?;
                        new_batches.push(batch);
                    }

                    // Swap batches: Remove existing batches
                    // Existing buffers will automatically be removed,
                    // buffers and other gpu resources are released by implementing the drop trait
                    self.batches = new_batches;
                    self.batch_thread_receiver = None;
                    self.is_dirty = false;
                    debug!("Finished cube_renderer update job");
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    // println!("Task still running...");
                }
                Err(err) => {
                    error!("Task sender was dropped unexpectedly: {err}");
                }
            }
        } else {
            // Update requested, but no thread started yet. Need to spin one up
            debug!("Starting cube_renderer update job");
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

    pub fn tick(&mut self, _dt: f32, camera_fov: &IAabb) {
        if self.is_dirty {
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
            debug!("Cannot fit entire chunk into current batch. Creating new batch");
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
    trace!(
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

        // Calculate light attributes
        let world_space_light_dir = Quat::from_rotation_x(20.0) * Vec3::Y;
        // NOTE: Could use self.color here for debugging
        let ambient_light_col = Vec3::ONE * 0.5;

        self.shader.use_program();
        self.shader.set_uniform_mat4("uView", &view);
        self.shader.set_uniform_mat4("uProjection", &projection);
        self.shader
            .set_uniform_vec3("uLightDir", &world_space_light_dir);
        self.shader
            .set_uniform_vec3("uAmbientLightColor", &ambient_light_col);

        for batch in &mut self.batches {
            batch.render(gl, self.vertex_count);
        }
    }
}

impl Drop for CubeRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vertex_position_vbo);
            self.gl.delete_buffer(self.vertex_normal_vbo);
            self.gl.delete_buffer(self.vertex_tex_coord_vbo);
        }
    }
}
