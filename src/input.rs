use std::collections::HashSet;
use winit::{event::MouseButton, keyboard::KeyCode};

pub struct InputState {
    pub keys_pressed: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_position: (f64, f64),
}

impl InputState {
    pub fn new() -> InputState {
        let keys_pressed = HashSet::<KeyCode>::new();
        let mouse_buttons_pressed = HashSet::<MouseButton>::new();
        Self {
            keys_pressed,
            mouse_buttons_pressed,
            mouse_position: (0.0, 0.0),
        }
    }

    pub fn key_pressed(&mut self, code: KeyCode) {
        self.keys_pressed.insert(code);
    }
    pub fn key_released(&mut self, code: &KeyCode) {
        self.keys_pressed.remove(code);
    }
    pub fn is_key_pressed(&self, code: &KeyCode) -> bool {
        self.keys_pressed.contains(code)
    }
    pub fn mouse_button_pressed(&mut self, button: MouseButton) {
        self.mouse_buttons_pressed.insert(button);
    }
    pub fn mouse_button_released(&mut self, button: &MouseButton) {
        self.mouse_buttons_pressed.remove(button);
    }
    pub fn register_mouse_delta(&mut self, delta: (f64, f64)) {
        self.mouse_position.0 += delta.0;
        self.mouse_position.1 += delta.1;
    }
    pub fn get_mouse_position_f32(&self) -> (f32, f32) {
        (self.mouse_position.0 as f32, self.mouse_position.1 as f32)
    }
    pub fn is_mouse_button_pressed(&self, btn: &MouseButton) -> bool {
        self.mouse_buttons_pressed.contains(btn)
    }
}
