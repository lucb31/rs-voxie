use std::sync::mpsc::Sender;

use hecs::World;

use crate::scenes::Scene;

use super::NetworkCommand;

pub trait NetworkScene: Scene {
    fn set_broadcast(&mut self, tx: Sender<NetworkCommand>);
    fn get_world(&mut self) -> &mut World;
    fn start_match(&mut self);
    fn game_over(&self) -> bool;
}
