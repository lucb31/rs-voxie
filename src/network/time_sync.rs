use std::time::{Duration, Instant};

pub struct TimeSync {
    /// Estimated offset: client_instant - server_time
    offset: Duration,
    last_update: Instant,
}

impl TimeSync {
    pub fn new() -> Self {
        Self {
            offset: Duration::ZERO,
            last_update: Instant::now(),
        }
    }

    /// Call this when a snapshot arrives
    pub fn update(
        &mut self,
        snapshot_server_time: Duration,
        receive_instant: Instant,
        rtt: Duration,
    ) {
        let estimated_server_now = snapshot_server_time + rtt / 2;
        let new_offset = receive_instant
            .checked_duration_since(Instant::now() - estimated_server_now)
            .unwrap_or(self.offset);

        // Exponential smoothing to avoid jitter
        let alpha = 0.1;
        self.offset = self.offset.mul_f32(1.0 - alpha) + new_offset.mul_f32(alpha);

        self.last_update = receive_instant;
    }

    pub fn server_time_at(&self, now: Instant) -> Duration {
        now.duration_since(self.last_update) + self.offset
    }
}
