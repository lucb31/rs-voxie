use std::time::{Duration, Instant};

const WINDOW_SECS: usize = 10;

#[derive(Debug)]
pub struct TrafficMeter {
    downstream: [u64; WINDOW_SECS],
    upstream: [u64; WINDOW_SECS],
    last_tick: Instant,
    current_index: usize,
}

impl TrafficMeter {
    pub fn new() -> Self {
        Self {
            downstream: [0; WINDOW_SECS],
            upstream: [0; WINDOW_SECS],
            last_tick: Instant::now(),
            current_index: 0,
        }
    }

    /// Advances the sliding window based on elapsed time
    fn advance_window(&mut self) {
        let elapsed_secs = self.last_tick.elapsed().as_secs() as usize;

        if elapsed_secs == 0 {
            return;
        }

        let steps = elapsed_secs.min(WINDOW_SECS);

        for _ in 0..steps {
            self.current_index = (self.current_index + 1) % WINDOW_SECS;
            self.downstream[self.current_index] = 0;
            self.upstream[self.current_index] = 0;
        }

        self.last_tick += Duration::from_secs(elapsed_secs as u64);
    }

    /// Record downstream traffic (bytes received)
    pub fn track_downstream(&mut self, size: usize) {
        self.advance_window();
        self.downstream[self.current_index] += size as u64;
    }

    /// Record upstream traffic (bytes sent)
    pub fn track_upstream(&mut self, size: usize) {
        self.advance_window();
        self.upstream[self.current_index] += size as u64;
    }

    pub fn downstream_bps(&self) -> u64 {
        self.downstream.iter().sum::<u64>() / WINDOW_SECS as u64
    }

    pub fn upstream_bps(&self) -> u64 {
        self.upstream.iter().sum::<u64>() / WINDOW_SECS as u64
    }
}
