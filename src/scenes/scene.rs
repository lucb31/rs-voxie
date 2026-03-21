use std::time::Duration;

use hecs::World;

use crate::cameras::camera::Camera;

pub trait Renderer {
    fn render(&mut self, cam: &Camera);
}

pub trait BaseScene {
    /// If returns ECS, simple single-pass ecs render pipeline will be used
    fn get_world(&self) -> Option<&World>;
    fn get_title(&self) -> String;
    fn tick(&mut self, dt: f32);
    // Perform any initialization logic the scene might need
    fn start(&mut self);
}

#[cfg(feature = "gui")]
pub trait GuiScene: BaseScene {
    fn get_stats(&self) -> super::SceneStats;
    fn render(&mut self, gl: &glow::Context, dt: Duration);
    fn render_ui(&mut self, ui: &mut imgui::Ui);
}
