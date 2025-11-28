use std::collections::HashSet;
use winit::{event::MouseButton, keyboard::KeyCode};

pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_position: (f64, f64),

    mouse_drag_delta: (f64, f64),
    mouse_drag_button: MouseButton,
}

impl InputState {
    pub fn new() -> InputState {
        let keys_pressed = HashSet::<KeyCode>::new();
        let mouse_buttons_pressed = HashSet::<MouseButton>::new();
        Self {
            keys_pressed,
            mouse_buttons_pressed,
            mouse_position: (0.0, 0.0),
            mouse_drag_delta: (0.0, 0.0),
            mouse_drag_button: MouseButton::Left,
        }
    }

    pub fn key_pressed(&mut self, code: KeyCode) {
        self.keys_pressed.insert(code);
    }
    pub fn key_released(&mut self, code: &KeyCode) {
        self.keys_pressed.remove(code);
    }
    pub fn mouse_button_pressed(&mut self, button: MouseButton) {
        self.mouse_buttons_pressed.insert(button);
    }
    pub fn mouse_button_released(&mut self, button: &MouseButton) {
        self.mouse_buttons_pressed.remove(button);
    }
    pub fn mouse_moved(&mut self, position: (f64, f64)) {
        if self.mouse_buttons_pressed.contains(&self.mouse_drag_button) {
            self.mouse_drag_delta.0 = self.mouse_position.0 - position.0;
            self.mouse_drag_delta.1 = self.mouse_position.1 - position.1;
        }
        self.mouse_position = position;
    }
    // Just and interims fix until we've figured out how multiple components
    // can consume the delta
    pub fn get_and_reset_mouse_moved(&mut self) -> (f64, f64) {
        let res = self.mouse_drag_delta;
        self.mouse_drag_delta = (0.0, 0.0);
        res
    }
    pub fn is_mouse_button_pressed(&self, btn: &MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(btn)
    }
}
