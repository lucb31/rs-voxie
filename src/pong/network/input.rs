use winit::keyboard::KeyCode;

use crate::{
    input::InputState,
    pong::network::client::{ClientMessage, InputSample},
};

const ACK_BUFFER_SIZE: usize = 60; // Stores input for up to 1s

impl Clone for InputSample {
    fn clone(&self) -> Self {
        Self {
            client_tick: self.client_tick,
            vertical_velocity: self.vertical_velocity,
        }
    }
}

pub(crate) struct ClientInputBuffer {
    last_acked_client_tick: u32,
    input_buffer: Vec<InputSample>,
}

impl ClientInputBuffer {
    pub fn new() -> ClientInputBuffer {
        Self {
            last_acked_client_tick: 0,
            input_buffer: Vec::with_capacity(ACK_BUFFER_SIZE),
        }
    }

    pub fn update_acked_client_tick(&mut self, acked_client_tick: u32) {
        self.last_acked_client_tick = acked_client_tick;
        self.input_buffer
            .retain(|sample| sample.client_tick > acked_client_tick);
    }

    pub fn set_buffer(&mut self, buffer: Vec<InputSample>) {
        self.input_buffer = buffer;
    }

    pub fn get_buffer_size(&self) -> usize {
        self.input_buffer.len()
    }

    pub fn get_last_acked(&self) -> u32 {
        self.last_acked_client_tick
    }

    /// Returns oldest client input sample
    pub fn get_oldest(&self) -> Option<&InputSample> {
        let mut sample_to_process: Option<&InputSample> = None;
        for sample in &self.input_buffer {
            if sample.client_tick <= self.last_acked_client_tick {
                continue;
            }
            if sample_to_process.is_none_or(|s| s.client_tick > sample.client_tick) {
                sample_to_process = Some(sample);
            }
        }
        sample_to_process
    }

    /// Returns most-recently added input sample
    pub fn last(&self) -> Option<&InputSample> {
        self.input_buffer.last()
    }

    pub fn sample_input(&mut self, input: &InputState, client_tick: u32) {
        let mut vertical_velocity = 0.0;
        if input.is_key_pressed(&KeyCode::KeyW) {
            vertical_velocity += 1.0;
        }
        if input.is_key_pressed(&KeyCode::KeyS) {
            vertical_velocity -= 1.0;
        }
        let sample = InputSample {
            client_tick,
            vertical_velocity,
        };
        self.input_buffer.push(sample);
    }

    pub fn assemble_input_sync_cmd(&self) -> ClientMessage {
        debug_assert!(
            self.input_buffer.len() < ACK_BUFFER_SIZE,
            "Input buffer overflow"
        );
        ClientMessage::InputSync {
            last_acked_client_tick: self.last_acked_client_tick,
            unacked_inputs: self.input_buffer.clone(),
        }
    }
}
