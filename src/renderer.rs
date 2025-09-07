use std::time::Instant;

use glam::Vec3;
use glow::HasContext;

use crate::{camera::Camera, cube::CubeRenderer, triangle::TriangleRenderer};

pub struct Renderer {
    last: Instant,
    pub camera: Camera,
    triangle: TriangleRenderer,
    cube: CubeRenderer,
}

impl Renderer {
    pub fn new(gl: &glow::Context) -> Renderer {
        let now = Instant::now();
        let mut camera = Camera::new();
        camera.set_velocity(Vec3::ZERO);

        Self {
            camera,
            cube: CubeRenderer::new(gl),
            triangle: TriangleRenderer::new(gl),
            last: now,
        }
    }

    pub fn render(&mut self, gl: &glow::Context) {
        // FIX: Currently processing and rendering will be done in one step
        // MAYBE this does not have to be mutable once we separate the steps
        self.process();

        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
        //self.triangle.render(gl);
        self.cube.render(gl, &self.camera);
    }

    pub fn process(&mut self) {
        let now = Instant::now();
        let dt = now.duration_since(self.last).as_secs_f32();
        self.camera.process(dt);
        self.last = now;
    }

    pub fn destroy(&self, gl: &glow::Context) {
        self.triangle.destroy(gl);
        self.cube.destroy(gl);
    }
}
