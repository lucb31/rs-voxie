use std::{
    fs::{File, OpenOptions, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
    time::Instant,
};

pub struct SceneStats {
    frame_count: u32,
    first: Instant,
    last: Instant,
    title: String,
    cube_count: u32,
}

impl SceneStats {
    pub fn new(
        frame_count: u32,
        first: Instant,
        last: Instant,
        title: String,
        cube_count: u32,
    ) -> SceneStats {
        Self {
            frame_count,
            first,
            last,
            title,
            cube_count,
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

    /// Initializes the CSV file by writing a header if it doesn't exist yet
    fn init_csv(&self, path: &str) -> Result<(), std::io::Error> {
        let path = Path::new(path);

        // Ensure parent directories exist
        if let Some(parent) = path.parent() {
            create_dir_all(parent)?;
        }

        // Only create the file if it doesn't exist
        if !path.exists() {
            let mut file = File::create(path)?;
            writeln!(file, "CubeCount,FrameCount,ElapsedSeconds,AvgFPS")?;
        }

        Ok(())
    }

    /// Appends the current scene's stats to the CSV file
    pub fn save_scene_stats(&self, path: &str) -> Result<(), std::io::Error> {
        // Ensure file exists
        self.init_csv(path)?;

        let elapsed = self.last.duration_since(self.first).as_secs_f32();
        let avg_fps = (self.frame_count as f32) / elapsed;
        let file = OpenOptions::new().append(true).create(true).open(path)?;
        let mut writer = BufWriter::new(file);

        writeln!(
            writer,
            "{},{},{:.3},{:.2}",
            self.cube_count, self.frame_count, elapsed, avg_fps
        )?;

        Ok(())
    }
}
