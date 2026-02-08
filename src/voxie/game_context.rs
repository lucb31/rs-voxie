use std::{cell::RefCell, rc::Rc, time::Instant};

use crate::input::InputState;

pub struct GameContext {
    pub input_state: Rc<RefCell<InputState>>,
    pub current_frame: u32,
    pub start_time: Instant,
}

impl GameContext {
    pub fn new(input_state: Rc<RefCell<InputState>>) -> GameContext {
        Self {
            input_state,
            current_frame: 0,
            start_time: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.current_frame += 1;
    }
}
