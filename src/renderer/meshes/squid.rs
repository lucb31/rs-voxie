use std::{error::Error, rc::Rc};

use glow::HasContext;

use crate::{meshes::objmesh::ObjMesh, renderer::shader::Shader};

use super::Mesh;

pub fn squid_mesh(gl: &Rc<glow::Context>) -> Result<Mesh, Box<dyn Error>> {
    let shader = Shader::new(
        gl,
        "assets/shaders/cube.vert",
        "assets/shaders/cube-diffuse.frag",
    )?;

    // Load vertex data from mesh
    let mut mesh = ObjMesh::new().with_blender_axis_fix(true);
    mesh.load("assets/squid_centered.obj")
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
