use std::{cell::RefCell, error::Error, rc::Rc};

use glam::{Quat, Vec3};
use glow::HasContext;

use crate::{
    cameras::camera::Camera,
    input::InputState,
    meshes::{cubemesh::CubeMesh, sphere::SphereMesh},
    scene::Scene,
};

/// Used to debug & visualize lighting shaders & algorithms
pub struct LightingScene {
    camera: Rc<RefCell<Camera>>,
    cube: CubeMesh,
    gl: Rc<glow::Context>,
    input_state: Rc<RefCell<InputState>>,
}

impl LightingScene {
    pub fn new(
        gl: Rc<glow::Context>,
        input_state: Rc<RefCell<InputState>>,
    ) -> Result<LightingScene, Box<dyn Error>> {
        let mut camera = Camera::new();
        camera.position = Vec3::new(0.0, 3.0, -2.5);

        let cube = CubeMesh::new(&gl)?;
        camera.look_at(cube.position);

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS);
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        let mut sphere = SphereMesh::new(gl.clone())?;
        sphere.position = Vec3::new(0.0, 0.0, -0.1);
        sphere.radius = 0.49;
        sphere.color = Vec3::new(0.0, 0.0, 1.0);

        Ok(Self {
            cube,
            camera: Rc::new(RefCell::new(camera)),
            gl,
            input_state,
        })
    }

    // Simple object rotation to mimic arcball
    fn process_mouse_movement(&mut self) {
        let mut input_state = self.input_state.borrow_mut();
        if !input_state.is_mouse_button_pressed(&winit::event::MouseButton::Left) {
            return;
        }
        let delta = input_state.get_and_reset_mouse_moved();
        let dx = delta.0 as f32;
        let dy = delta.1 as f32;
        let sensitivity = 0.01;
        let yaw = Quat::from_rotation_y(dx * sensitivity);
        let pitch = Quat::from_rotation_x(dy * sensitivity);
        self.cube.rotation *= yaw * pitch;
    }
}

impl Scene for LightingScene {
    fn get_title(&self) -> String {
        "Lighting Test".to_string()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    fn get_stats(&self) -> crate::scenes::SceneStats {
        todo!()
    }

    fn tick(&mut self, dt: f32, gl: &glow::Context) {
        self.process_mouse_movement();
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
        self.cube.render(gl, &self.camera.borrow());
    }

    fn render_ui(&mut self, ui: &mut imgui::Ui) {}

    fn start(&mut self) {}
}
