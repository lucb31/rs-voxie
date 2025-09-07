use glow::HasContext;

use crate::{cube::CubeRenderer, triangle::TriangleRenderer};

pub struct Renderer {
    triangle: TriangleRenderer,
    cube: CubeRenderer,
}

impl Renderer {
    pub fn new(gl: &glow::Context) -> Renderer {
        Self {
            cube: CubeRenderer::new(gl),
            triangle: TriangleRenderer::new(gl),
        }
    }

    pub fn render(&self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        //self.triangle.render(gl);
        self.cube.render(gl);
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.triangle.destroy(gl);
        self.cube.destroy(gl);
    }
}
