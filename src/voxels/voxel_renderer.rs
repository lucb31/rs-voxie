use std::{
    cell::RefCell, collections::HashMap, error::Error, path::Path, rc::Rc, sync::Arc, time::Instant,
};

use glam::{IVec3, Quat, Vec3};
use glow::{HasContext, NativeBuffer};
use log::{debug, error, trace};

use crate::{
    cameras::camera::Camera,
    meshes::objmesh::ObjMesh,
    octree::IAabb,
    renderer::{shader::Shader, texture::Texture},
    voxel::{CHUNK_SIZE, VoxelChunk, VoxelKind},
    world::VoxelWorld,
};

const CAMERA_FOV_RADIUS: i32 = 4;

pub struct VoxelWorldRenderer {
    // Common rendering resources shared across chunk meshes
    gl: Rc<glow::Context>,
    texture: Texture,
    shader: Shader,
    vertex_position_vbo: NativeBuffer,
    vertex_normal_vbo: NativeBuffer,
    vertex_tex_coord_vbo: NativeBuffer,
    vertex_count: usize,

    world: Rc<RefCell<VoxelWorld>>,
    // Hash map so we can easily access and replace chunk meshes at given position
    // Contains only chunks within current FoV
    chunk_meshes: HashMap<IVec3, Arc<VoxelChunkMesh>>,
    // Rendering volume in which chunk meshes will be generated and rendered
    render_bb: IAabb,
    // Optimization helper
    last_render_bb: IAabb,
}

impl VoxelWorldRenderer {
    pub fn new(
        gl: Rc<glow::Context>,
        world: Rc<RefCell<VoxelWorld>>,
    ) -> Result<VoxelWorldRenderer, Box<dyn Error>> {
        // Setup shader
        let shader = Shader::new(
            gl.clone(),
            "assets/shaders/voxel.vert",
            "assets/shaders/cube-diffuse.frag",
        )?;

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
            // Load texture
            let texture = Texture::new(&gl, Path::new("assets/textures/dirt.png"))
                .expect("Could not load texture");

            Ok(Self {
                chunk_meshes: HashMap::new(),
                gl: gl.clone(),
                last_render_bb: IAabb::new(&IVec3::ONE, 1),
                render_bb: IAabb::new(&IVec3::ONE, 1),
                shader,
                texture,
                vertex_count,
                vertex_normal_vbo: normals_vbo,
                vertex_position_vbo: positions_vbo,
                vertex_tex_coord_vbo: tex_coords_vbo,
                world,
            })
        }
    }

    pub fn get_instance_count(&self) -> i32 {
        let mut count = 0;
        for (_, mesh) in self.chunk_meshes.iter() {
            count += mesh.instance_count;
        }
        count
    }

    pub fn render_ui(&mut self, ui: &mut imgui::Ui) {
        // Get display size
        let display_size = ui.io().display_size;
        let window_size = [200.0, 100.0];
        // Compute top-right position
        let pos = [display_size[0] - window_size[0], 0.0];
        ui.window("Voxels")
            .size(window_size, imgui::Condition::FirstUseEver)
            .position(pos, imgui::Condition::FirstUseEver)
            .build(|| {
                ui.text(format!(
                    "Visible chunks: {}",
                    self.get_visible_meshes().len()
                ));
                ui.text(format!(
                    "Rendered cubes: {}",
                    format_with_commas(self.get_instance_count() as u64)
                ));
            });
    }

    pub fn tick(&mut self, _dt: f32, camera_pos: &Vec3) {
        // Chunk-grid snapped camera pos
        let render_bb_min = IVec3::new(
            ((camera_pos.x / CHUNK_SIZE as f32) as i32 - CAMERA_FOV_RADIUS) * CHUNK_SIZE as i32,
            ((camera_pos.y / CHUNK_SIZE as f32) as i32 - CAMERA_FOV_RADIUS) * CHUNK_SIZE as i32,
            ((camera_pos.z / CHUNK_SIZE as f32) as i32 - CAMERA_FOV_RADIUS) * CHUNK_SIZE as i32,
        );
        let render_bb_max = IVec3::new(
            ((camera_pos.x / CHUNK_SIZE as f32) as i32 + CAMERA_FOV_RADIUS) * CHUNK_SIZE as i32,
            ((camera_pos.y / CHUNK_SIZE as f32) as i32 + CAMERA_FOV_RADIUS) * CHUNK_SIZE as i32,
            ((camera_pos.z / CHUNK_SIZE as f32) as i32 + CAMERA_FOV_RADIUS) * CHUNK_SIZE as i32,
        );
        self.render_bb = IAabb::new_rect(render_bb_min, render_bb_max);
        self.update_chunk_meshes();
    }

    fn update_chunk_meshes(&mut self) {
        // Optimization: If render_bb did not move, nothing to do
        if self.render_bb == self.last_render_bb {
            return;
        }

        // Query world
        let start_mesh_update = Instant::now();
        let chunks_within_render_bb = self.world.borrow().query_region_chunks(&self.render_bb);
        // Generate new hash map, reusing existing meshes where possible
        // Justification to assemble new map instead of inplace:
        // - VoxelChunkMesh does not take a lot of memory
        // - We might need to offload this to a separate thread
        let mut new_chunk_map: HashMap<IVec3, Arc<VoxelChunkMesh>> =
            HashMap::with_capacity(chunks_within_render_bb.len());
        let mut meshes_generated = 0;
        for chunk in &chunks_within_render_bb {
            // Optimization: Do not generate meshes for already visible chunks
            let existing_mesh = self.chunk_meshes.get(&chunk.position);
            match existing_mesh {
                Some(mesh) => {
                    new_chunk_map.insert(chunk.position, mesh.clone());
                }
                None => {
                    let mesh_result = VoxelChunkMesh::new(
                        self.gl.clone(),
                        self.vertex_position_vbo,
                        self.vertex_normal_vbo,
                        self.vertex_tex_coord_vbo,
                        chunk,
                    );
                    match mesh_result {
                        Ok(mesh) => {
                            new_chunk_map.insert(chunk.position, Arc::new(mesh));
                        }
                        Err(err) => error!("Unable to generate voxel chunk mesh: {err}"),
                    }
                    meshes_generated += 1;
                }
            }
        }
        debug!(
            "Updated chunk meshes. {} chunks within render BB {:?}. Generated {} new meshes. Took {} ms",
            chunks_within_render_bb.len(),
            self.render_bb,
            meshes_generated,
            start_mesh_update.elapsed().as_secs_f32() * 1000.0
        );
        self.chunk_meshes = new_chunk_map;
        self.last_render_bb = self.render_bb.clone();
    }

    fn get_visible_meshes(&self) -> Vec<&VoxelChunkMesh> {
        // Will put camera frustum culling here
        // Interims solution: Returns all meshes
        let mut res: Vec<&VoxelChunkMesh> = Vec::with_capacity(self.chunk_meshes.len());
        let mut skipped_chunks = 0;
        for (_, chunk) in self.chunk_meshes.iter() {
            // We can skip empty meshes
            if chunk.instance_count > 0 {
                res.push(chunk);
            } else {
                skipped_chunks += 1;
            }
        }
        // trace!("Skipped {skipped_chunks} empty chunks in visibility check");
        res
    }

    pub fn render(&mut self, cam: &Camera) {
        let view = cam.get_view_matrix();
        let projection = cam.get_projection_matrix();

        // Calculate light attributes
        let world_space_light_dir = Quat::from_rotation_x(20.0) * Vec3::Y;
        let ambient_light_col = Vec3::ONE * 0.5;

        self.shader.use_program();
        self.shader.set_uniform_mat4("uView", &view);
        self.shader.set_uniform_mat4("uProjection", &projection);
        self.shader
            .set_uniform_vec3("uLightDir", &world_space_light_dir);
        self.shader
            .set_uniform_vec3("uAmbientLightColor", &ambient_light_col);
        self.texture.bind();

        let visible_meshes = self.get_visible_meshes();
        //         trace!(
        //             "Starting to render meshes: {} of {} visible",
        //             visible_meshes.len(),
        //             self.chunk_meshes.len(),
        //         );
        for mesh in visible_meshes {
            unsafe {
                self.gl.bind_vertex_array(Some(mesh.vao));
                self.gl.draw_arrays_instanced(
                    glow::TRIANGLES,
                    0,
                    self.vertex_count as i32,
                    mesh.instance_count,
                );
                self.gl.bind_vertex_array(None);
            }
        }
        self.texture.unbind();
    }
}

impl Drop for VoxelWorldRenderer {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.vertex_position_vbo);
            self.gl.delete_buffer(self.vertex_normal_vbo);
            self.gl.delete_buffer(self.vertex_tex_coord_vbo);
        }
    }
}

struct VoxelChunkMesh {
    gl: Rc<glow::Context>,
    vao: <glow::Context as HasContext>::VertexArray,
    // Voxel position buffer in this chunk
    instance_vbo: NativeBuffer,
    // Number of voxels rendered
    pub instance_count: i32,
}

impl VoxelChunkMesh {
    pub fn new(
        gl: Rc<glow::Context>,
        vertex_position_vbo: NativeBuffer,
        vertex_normal_vbo: NativeBuffer,
        vertex_tex_coords_vbo: NativeBuffer,
        chunk: &VoxelChunk,
    ) -> Result<VoxelChunkMesh, Box<dyn Error>> {
        // TODO: Deprecate the position prop of voxel
        let mut positions: Vec<Vec3> = Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for voxel in chunk.voxel_slice() {
            if matches!(voxel.kind, VoxelKind::Air) {
                continue;
            }
            positions.push(voxel.position);
        }
        let positons_bytes: &[u8] = bytemuck::cast_slice(&positions);

        // Setup buffers and vertex attributes
        unsafe {
            let start_buffering = Instant::now();
            // Buffer vertex position data
            let instance_vbo = gl.create_buffer().expect("Cannot create instance vbo");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(instance_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positons_bytes, gl::STATIC_DRAW);

            // Setup vertex array object
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

            // Cleanup
            gl.bind_buffer(gl::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            trace!(
                "Chunk GPU buffering of {} instances took {}s",
                positions.len(),
                start_buffering.elapsed().as_secs_f32()
            );
            Ok(Self {
                gl,
                instance_count: positions.len() as i32,
                instance_vbo,
                vao,
            })
        }
    }
}
impl Drop for VoxelChunkMesh {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_buffer(self.instance_vbo);
            self.gl.delete_vertex_array(self.vao);
        }
    }
}

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let mut chars = s.chars().rev().enumerate();
    while let Some((i, c)) = chars.next() {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
