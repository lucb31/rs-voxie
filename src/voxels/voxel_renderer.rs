use std::{collections::HashMap, error::Error, mem::offset_of, path::Path, rc::Rc, time::Instant};

use bytemuck::{Pod, Zeroable};
use glam::{IVec3, Quat, Vec3};
use glow::{HasContext, NativeBuffer};
use log::{debug, error, trace};

use crate::{
    cameras::camera::Camera,
    meshes::objmesh::ObjMesh,
    octree::IAabb,
    renderer::{shader::Shader, texture::Texture},
    scenes::metrics::SimpleMovingAverage,
    voxels::{CHUNK_SIZE, VoxelChunk, VoxelKind, VoxelWorld},
};

const CAMERA_FOV_RADIUS: i32 = 4;

struct VoxelRendererDebugInfo {
    visible_voxels: i32,
    visible_chunks: usize,
    chunks_within_render_bb: usize,
    render_time: SimpleMovingAverage,
}

impl VoxelRendererDebugInfo {
    pub fn new() -> VoxelRendererDebugInfo {
        Self {
            visible_voxels: 0,
            visible_chunks: 0,
            chunks_within_render_bb: 0,
            render_time: SimpleMovingAverage::new(100),
        }
    }
}

pub struct VoxelWorldRenderer {
    // Common rendering resources shared across chunk meshes
    gl: Rc<glow::Context>,
    texture: Texture,
    shader: Shader,
    vertex_position_vbo: NativeBuffer,
    vertex_normal_vbo: NativeBuffer,
    vertex_tex_coord_vbo: NativeBuffer,
    vertex_count: usize,

    // Hash map so we can easily access and replace chunk meshes at given position
    // Contains only chunks within current FoV
    chunk_meshes: HashMap<IVec3, Rc<VoxelChunkMesh>>,
    // Rendering volume in which chunk meshes will be generated and rendered
    render_bb: IAabb,

    debug_info: VoxelRendererDebugInfo,
}

impl VoxelWorldRenderer {
    pub fn new(gl: &Rc<glow::Context>) -> Result<VoxelWorldRenderer, Box<dyn Error>> {
        // Setup shader
        let shader = Shader::new(
            gl,
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
            let texture = Texture::new(gl, Path::new("assets/textures/atlas.png"))
                .expect("Could not load texture");

            Ok(Self {
                chunk_meshes: HashMap::new(),
                debug_info: VoxelRendererDebugInfo::new(),
                gl: Rc::clone(gl),
                render_bb: IAabb::new(&IVec3::ONE, 1),
                shader,
                texture,
                vertex_count,
                vertex_normal_vbo: normals_vbo,
                vertex_position_vbo: positions_vbo,
                vertex_tex_coord_vbo: tex_coords_vbo,
            })
        }
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
                    "Chunks within region: {}",
                    self.debug_info.chunks_within_render_bb
                ));
                ui.text(format!(
                    "Visible voxel meshes: {}",
                    self.debug_info.visible_chunks
                ));
                ui.text(format!(
                    "Rendered cubes: {}",
                    format_with_commas(self.debug_info.visible_voxels as u64)
                ));
                ui.text(format!(
                    "Time to render: {:.0}ns",
                    self.debug_info.render_time.get(),
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
    }

    fn get_visible_chunks(
        &mut self,
        cam: &Camera,
        world: &VoxelWorld,
    ) -> impl Iterator<Item = Rc<VoxelChunkMesh>> {
        let camera_frustum = cam.get_frustum();
        world
            .iter_region_chunks(&self.render_bb)
            .filter(move |chunk| {
                // Frustum culling
                let chunk_bb = chunk.get_bb_i();
                camera_frustum.contains_aabb(&chunk_bb)
            })
            .filter_map(|chunk| {
                // Optimization: Do not generate meshes for already meshed chunks that are **not**
                // dirty
                if !chunk.is_dirty()
                    && let Some(mesh) = self.chunk_meshes.get(&chunk.position)
                {
                    // Skip empty meshes
                    if mesh.instance_count == 0 {
                        return None;
                    }
                    return Some(Rc::clone(mesh));
                }
                match VoxelChunkMesh::new(
                    &self.gl,
                    self.vertex_position_vbo,
                    self.vertex_normal_vbo,
                    self.vertex_tex_coord_vbo,
                    chunk,
                ) {
                    Ok(mesh) => {
                        let rc_mesh = Rc::new(mesh);
                        self.chunk_meshes
                            .insert(chunk.position, Rc::clone(&rc_mesh));
                        chunk.set_clean();
                        Some(rc_mesh)
                    }
                    Err(err) => {
                        error!("Unable to generate voxel chunk mesh: {err}");
                        None
                    }
                }
            })
    }

    pub fn render(&mut self, cam: &Camera, world: &VoxelWorld) {
        let start_timestamp = Instant::now();
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

        let gl = Rc::clone(&self.gl);
        let vertex_count = self.vertex_count;
        let visible_meshes = self.get_visible_chunks(cam, world);
        let mut count_voxels = 0;
        let mut count_chunks = 0;
        for mesh in visible_meshes {
            unsafe {
                gl.bind_vertex_array(Some(mesh.vao));
                gl.draw_arrays_instanced(
                    glow::TRIANGLES,
                    0,
                    vertex_count as i32,
                    mesh.instance_count,
                );
                gl.bind_vertex_array(None);
            }
            count_voxels += mesh.instance_count;
            count_chunks += 1;
        }
        self.texture.unbind();

        self.debug_info.visible_voxels = count_voxels;
        self.debug_info.visible_chunks = count_chunks;
        self.debug_info.render_time.add_elapsed(start_timestamp);
        debug!(
            "Voxel render took {}ms",
            start_timestamp.elapsed().as_secs_f32() * 1e3
        );
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

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ChunkVertexData {
    position: Vec3,
    material_index: u32,
}
impl VoxelChunkMesh {
    pub fn new(
        gl: &Rc<glow::Context>,
        vertex_position_vbo: NativeBuffer,
        vertex_normal_vbo: NativeBuffer,
        vertex_tex_coords_vbo: NativeBuffer,
        chunk: &VoxelChunk,
    ) -> Result<VoxelChunkMesh, Box<dyn Error>> {
        let mut vertex_data: Vec<ChunkVertexData> =
            Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE);
        for voxel in chunk.voxel_slice() {
            if matches!(voxel.kind, VoxelKind::Air) {
                continue;
            }
            vertex_data.push(ChunkVertexData {
                position: voxel.position,
                material_index: voxel.kind.material_index(),
            });
        }
        let vertex_data_bytes: &[u8] = bytemuck::cast_slice(&vertex_data);

        // Setup buffers and vertex attributes
        unsafe {
            let start_buffering = Instant::now();
            // Buffer vertex position data
            let instance_vbo = gl.create_buffer().expect("Cannot create instance vbo");
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(instance_vbo));
            gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_data_bytes, gl::STATIC_DRAW);

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
            // Setup tex_coords attribute
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(vertex_tex_coords_vbo));
            gl.vertex_attrib_pointer_f32(3, 2, gl::FLOAT, false, 0, 0);
            gl.enable_vertex_array_attrib(vao, 3);

            // Setup vertex instance buffer
            gl.bind_buffer(gl::ARRAY_BUFFER, Some(instance_vbo));
            let stride = size_of::<ChunkVertexData>() as i32;
            // location attribute
            gl.vertex_attrib_pointer_f32(2, 3, gl::FLOAT, false, stride, 0);
            gl.enable_vertex_attrib_array(2);
            // Update vertex attribute at index 2 on every new instance
            gl.vertex_attrib_divisor(2, 1);
            // material index attribute
            gl.vertex_attrib_pointer_i32(
                4,
                1,
                gl::INT,
                stride,
                offset_of!(ChunkVertexData, material_index) as i32,
            );
            gl.enable_vertex_attrib_array(4);
            // Update vertex attribute at index 4 on every new instance
            gl.vertex_attrib_divisor(4, 1);

            // Cleanup
            gl.bind_buffer(gl::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            trace!(
                "Chunk GPU buffering of {} instances took {}s",
                vertex_data.len(),
                start_buffering.elapsed().as_secs_f32()
            );
            Ok(Self {
                gl: Rc::clone(gl),
                instance_count: vertex_data.len() as i32,
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
    let chars = s.chars().rev().enumerate();
    for (i, c) in chars {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}
