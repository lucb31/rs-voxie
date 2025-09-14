use std::time::Instant;

pub struct SceneStats {
    frame_count: u32,
    first: Instant,
    last: Instant,
    title: String,
}

impl SceneStats {
    pub fn new(frame_count: u32, first: Instant, last: Instant, title: String) -> SceneStats {
        Self {
            frame_count,
            first,
            last,
            title,
        }
    }

    pub fn print_scene_stats(&self) {
        let elapsed = self.last.duration_since(self.first).as_secs_f32();
        let avg_fps = (self.frame_count as f32) / elapsed;
        println!(
            "{}: Total frames drawn: {}, Time elapsed between first and last frame: {}, Avg fps: {} \n ",
            self.title, self.frame_count, elapsed, avg_fps
        )
    }
}
