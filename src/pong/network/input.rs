use crate::pong::network::client::InputSample;

pub(crate) const ACK_BUFFER_SIZE: usize = 60; // Stores input for up to 1s

impl Clone for InputSample {
    fn clone(&self) -> Self {
        Self {
            client_tick: self.client_tick,
            vertical_velocity: self.vertical_velocity,
        }
    }
}

pub(crate) struct ClientInputBuffer {
    pub(crate) last_acked_client_tick: u32,
    pub(crate) input_buffer: Vec<InputSample>,
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
}
