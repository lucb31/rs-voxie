use std::{
    cell::RefCell,
    error::Error,
    fs::{File, OpenOptions, create_dir_all},
    io::{BufWriter, Write},
    path::Path,
    rc::Rc,
    time::Instant,
};

use glam::{IVec3, Quat, Vec3};
use glow::HasContext;
use log::info;

use super::{Renderer, Scene};
use crate::{
    cameras::camera::Camera,
    cube::CubeRenderer,
    octree::IAabb,
    voxels::{CHUNK_SIZE, VoxelWorld},
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
        info!(
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

pub struct BenchmarkScene {
    pub title: String,

    pub start: Instant,
    pub last: Instant,
    pub camera: Rc<RefCell<Camera>>,
    // Rethink. We might not even need this
    gl: Rc<glow::Context>,

    world: Rc<RefCell<VoxelWorld>>,
    cube_renderer: CubeRenderer,

    cube_count: usize,
    frame_count: u32,
}

impl BenchmarkScene {
    pub fn new(
        gl: &Rc<glow::Context>,
        world_size: usize,
    ) -> Result<BenchmarkScene, Box<dyn Error>> {
        let now = Instant::now();
        let mut camera = Camera::new();
        camera.position = Vec3::new(58.0, 37.0, 53.0);
        camera.set_rotation(
            Quat::from_rotation_y(45f32.to_radians()) * Quat::from_rotation_x(-25f32.to_radians()),
        );

        // Setup cube world
        let world = Rc::new(RefCell::new(VoxelWorld::new_cubic(world_size)));
        let cube_renderer = CubeRenderer::new(gl, Rc::clone(&world))?;

        // Setup context
        unsafe {
            gl.enable(gl::CULL_FACE);
            gl.enable(gl::DEPTH_TEST);
            gl.depth_func(gl::LESS); // Default: Pass if the incoming depth is less than the stored depth
            gl.cull_face(gl::BACK);
            gl.front_face(gl::CCW);
        }

        Ok(Self {
            camera: Rc::new(RefCell::new(camera)),
            cube_count: world_size * world_size * world_size,
            cube_renderer,
            frame_count: 0,
            gl: Rc::clone(gl),
            last: now,
            start: now,
            title: "Unnamed scene".to_string(),
            world,
        })
    }
}

impl Scene for BenchmarkScene {
    fn render_ui(&mut self, _ui: &mut imgui::Ui) {}

    fn render(&mut self) {
        let gl = &self.gl;
        unsafe {
            gl.clear_color(0.05, 0.05, 0.1, 1.0);
            gl.clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        self.cube_renderer.render(&self.camera.borrow());
        self.frame_count += 1;
    }

    fn tick(&mut self, dt: f32) {
        let now = Instant::now();
        let camera_fov = IAabb::new(
            &IVec3::ZERO,
            self.world.borrow().get_size() * CHUNK_SIZE * 2,
        );
        self.cube_renderer.tick(dt, &camera_fov);
        self.last = now;
    }

    fn start(&mut self) {
        self.start = Instant::now();
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn get_main_camera(&self) -> Rc<RefCell<Camera>> {
        self.camera.clone()
    }

    fn get_stats(&self) -> SceneStats {
        SceneStats::new(
            self.frame_count,
            self.start,
            self.last,
            self.title.to_string(),
            self.cube_count as u32,
        )
    }
}
