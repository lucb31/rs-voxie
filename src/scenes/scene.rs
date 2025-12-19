use std::{cell::RefCell, rc::Rc};

use imgui::Ui;

use crate::{cameras::camera::Camera, scenes::SceneStats};

pub trait Renderer {
    fn render(&mut self, cam: &Camera);
}

pub trait Scene {
    fn get_title(&self) -> String;
    fn get_stats(&self) -> SceneStats;
    fn tick(&mut self, dt: f32);
    fn render(&mut self);
    fn render_ui(&mut self, ui: &mut Ui);
    // Perform any initialization logic the scene might need
    fn start(&mut self);
}
