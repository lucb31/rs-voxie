use std::{cell::RefCell, rc::Rc};

use crate::input::InputState;

pub struct GameContext {
    pub input_state: Rc<RefCell<InputState>>,
    pub current_frame: u32,
}

impl GameContext {
    pub fn new(input_state: Rc<RefCell<InputState>>) -> GameContext {
        Self {
            input_state,
            current_frame: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current_frame += 1;
    }
}
