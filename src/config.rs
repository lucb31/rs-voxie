use std::time::Duration;

pub const RESOLUTION_WIDTH: u32 = 1920;
pub const RESOLUTION_HEIGHT: u32 = 1080;
pub const SIMULATION_DT: Duration = Duration::from_nanos(1_000_000_000 / 60); // 60Hz
pub const BROADCAST_DT: Duration = Duration::from_nanos(1_000_000_000 / 30); // 30Hz
pub const USE_VSYNC: bool = true;
