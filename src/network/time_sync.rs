use std::time::{Duration, Instant};

use log::debug;

pub struct TimeSync {
    last_update_server_time: Duration,
    last_update_client_instant: Instant,
}

impl TimeSync {
    pub fn new() -> Self {
        Self {
            last_update_server_time: Duration::ZERO,
            last_update_client_instant: Instant::now(),
        }
    }

    pub fn update(
        &mut self,
        snapshot_server_time: Duration,
        receive_instant: Instant,
        rtt: Duration,
    ) {
        let estimated_server_now = snapshot_server_time + rtt / 2;
        // Exp smoothing to avoid jitter
        let alpha = 0.1;
        self.last_update_server_time =
            self.last_update_server_time.mul_f32(1.0 - alpha) + estimated_server_now.mul_f32(alpha);

        self.last_update_client_instant = receive_instant;
        debug!(
            "Time sync update: serverTime {:?}, receiveInstant {:?}, offset is now {:?} ",
            snapshot_server_time, receive_instant, self.last_update_server_time
        );
    }

    pub fn server_time_at(&self, now: Instant) -> Duration {
        self.last_update_server_time + now.duration_since(self.last_update_client_instant)
    }
}
