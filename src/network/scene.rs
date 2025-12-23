use hecs::World;

use crate::scenes::Scene;

pub trait NetworkScene: Scene {
    fn get_world(&mut self) -> &mut World;
    fn start_match(&mut self);
    fn game_over(&self) -> bool;
}
