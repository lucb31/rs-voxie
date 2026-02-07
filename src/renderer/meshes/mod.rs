pub(super) mod squid;

use std::{error::Error, rc::Rc};

use glow::HasContext;

use crate::meshes::objmesh::ObjMesh;

use super::{Mesh, shader::Shader};

pub(super) fn projectile_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/projectile.vert",
        "assets/shaders/sphere_rt.frag",
    )?;
    // Load vertex data from mesh
    let mut mesh = ObjMesh::new();
    mesh.load("assets/cube.obj").expect("Could not load mesh");
    let vertex_positions = mesh.get_vertex_buffers().position_buffer;
    let vertex_bytes: &[u8] = bytemuck::cast_slice(&vertex_positions);
    unsafe {
        // Setup vertex & index array and buffer
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        // Bind vertex data
        let vbo = gl.create_buffer()?;
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(
            0,
            3,
            gl::FLOAT,
            false,
            3 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_array_attrib(vao, 0);
        gl.bind_buffer(gl::ARRAY_BUFFER, None);
        // 3 because vertex pos has 3 coordinates for each vertex
        Ok(Mesh::new(shader, vao, (vertex_positions.len() / 3) as i32))
    }
}

pub(super) fn mesh_cube(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(gl, "assets/shaders/cube.vert", "assets/shaders/quad.frag")?;

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
        // Setup vertex & index array and buffer
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        // Buffer position data
        let positions_vbo = gl.create_buffer().expect("Cannot create buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positions_bytes, gl::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(0, 3, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 0);
        // Buffer normal data
        let normals_vbo = gl
            .create_buffer()
            .expect("Cannot create buffer for normals");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normals_bytes, gl::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(1, 3, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 1);
        // Buffer texture coordinate data
        let tex_coords_vbo = gl.create_buffer().expect("Cannot create buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(tex_coords_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, tex_coords_bytes, gl::STATIC_DRAW);
        gl.vertex_attrib_pointer_f32(3, 2, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 3);
        gl.bind_buffer(gl::ARRAY_BUFFER, None);

        Ok(Mesh::new(shader, vao, vertex_count as i32))
    }
}

pub(super) fn projectile2d_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/projectile_2d.vert",
        "assets/shaders/projectile_2d.frag",
    )?;
    // Load vertex data from mesh
    let mut mesh = ObjMesh::new();
    mesh.load("assets/cube.obj").expect("Could not load mesh");
    let vertex_positions = mesh.get_vertex_buffers().position_buffer;
    let vertex_bytes: &[u8] = bytemuck::cast_slice(&vertex_positions);
    unsafe {
        // Setup vertex & index array and buffer
        let vao = gl.create_vertex_array()?;
        gl.bind_vertex_array(Some(vao));
        // Bind vertex data
        let vbo = gl.create_buffer()?;
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, vertex_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(
            0,
            3,
            gl::FLOAT,
            false,
            3 * std::mem::size_of::<f32>() as i32,
            0,
        );
        gl.enable_vertex_array_attrib(vao, 0);
        gl.bind_buffer(gl::ARRAY_BUFFER, None);
        // 3 because vertex pos has 3 coordinates for each vertex
        Ok(Mesh::new(shader, vao, (vertex_positions.len() / 3) as i32))
    }
}

// Better than placing this randomly and having interdependencies between ecsrenderer and
// mesh implementations would be an asset manager that keeps track of meshes and allows registering
// / loading meshes
pub(super) fn player_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/cube.vert",
        "assets/shaders/cube-diffuse.frag",
    )?;

    // Load vertex data from mesh
    let mut mesh = ObjMesh::new().with_blender_axis_fix(true);
    mesh.load("assets/fish_centered.obj")
        .expect("Could not load mesh");
    let vertex_buffers = mesh.get_vertex_buffers();
    // NOTE: /3 because we have 3 coordinates per vertex
    let vertex_count = vertex_buffers.position_buffer.len() / 3;
    let positions_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.position_buffer);
    let normals_bytes: &[u8] = bytemuck::cast_slice(&vertex_buffers.normal_buffer);

    unsafe {
        // Setup vertex array object
        let vao = gl
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.bind_vertex_array(Some(vao));

        // Buffer position data
        let positions_vbo = gl.create_buffer().expect("Cannot create buffer");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(positions_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, positions_bytes, gl::STATIC_DRAW);
        // Setup position attribute
        gl.vertex_attrib_pointer_f32(0, 3, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 0);

        // Buffer normal data
        let normals_vbo = gl
            .create_buffer()
            .expect("Cannot create buffer for normals");
        gl.bind_buffer(gl::ARRAY_BUFFER, Some(normals_vbo));
        gl.buffer_data_u8_slice(gl::ARRAY_BUFFER, normals_bytes, gl::STATIC_DRAW);
        // Setup normal attribute
        gl.vertex_attrib_pointer_f32(1, 3, gl::FLOAT, false, 0, 0);
        gl.enable_vertex_array_attrib(vao, 1);

        // Cleanup
        gl.bind_buffer(gl::ARRAY_BUFFER, None);
        gl.bind_vertex_array(None);

        Ok(Mesh::new(shader, vao, vertex_count as i32))
    }
}
