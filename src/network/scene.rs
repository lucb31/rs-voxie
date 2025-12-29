pub trait ServerScene {
    fn tick(&mut self, dt: f32);
    fn broadcast_state(&self);
    fn get_title(&self) -> String;
}
