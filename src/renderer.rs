use glow::HasContext;

use crate::triangle::TriangleRenderer;

pub struct Renderer {
    triangle: TriangleRenderer,
}

impl Renderer {
    pub fn new(gl: &glow::Context) -> Renderer {
        Self {
            triangle: TriangleRenderer::new(gl),
        }
    }

    pub fn render(&self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        self.triangle.render(gl);
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.triangle.destroy(gl);
    }
}
