// Derived from https://github.com/imgui-rs/imgui-glow-renderer/blob/main/examples/glow_01_basic.rs
use std::{
    cell::RefCell,
    collections::VecDeque,
    error::Error,
    num::NonZeroU32,
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, SwapInterval, WindowSurface},
};
use imgui::Context;
use imgui_glow_renderer::AutoRenderer;
use imgui_winit_support::{
    WinitPlatform,
    winit::{
        dpi::LogicalSize,
        event_loop::EventLoop,
        window::{Window, WindowAttributes},
    },
};
use log::{error, info};
use raw_window_handle::HasWindowHandle;
use winit::{application::ApplicationHandler, keyboard::KeyCode};

use crate::{
    input::InputState,
    scenes::{Scene, metrics::SceneMetrics},
};

const USE_VSYNC: bool = true;

pub struct Application {
    pub max_scene_duration_secs: f32,

    pub input_state: Rc<RefCell<InputState>>,
    // Rendering & application loop context
    event_loop: Option<EventLoop<()>>,
    window: Window,
    surface: Surface<WindowSurface>,
    winit_platform: WinitPlatform,
    glutin_context: PossiblyCurrentContext,
    imgui_context: Context,
    ig_renderer: AutoRenderer,

    active_scene: Option<Box<dyn Scene>>,
    active_scene_started_at: Option<Instant>,
    available_scenes: VecDeque<Box<dyn Scene>>,

    current_frame_start: Instant,
    prev_frame_start: Instant,

    metrics: SceneMetrics,
}

impl ApplicationHandler for Application {
    fn resumed(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {}

    fn new_events(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _cause: winit::event::StartCause,
    ) {
        let now = Instant::now();
        let duration_since = now.duration_since(self.current_frame_start);
        self.imgui_context
            .io_mut()
            .update_delta_time(duration_since);
        self.prev_frame_start = self.current_frame_start;
        self.current_frame_start = now
    }

    fn about_to_wait(&mut self, _event_loop: &winit::event_loop::ActiveEventLoop) {
        self.winit_platform
            .prepare_frame(self.imgui_context.io_mut(), &self.window)
            .unwrap();
        self.window.request_redraw();
    }

    fn device_event(
        &mut self,
        _event_loop: &winit::event_loop::ActiveEventLoop,
        _device_id: winit::event::DeviceId,
        event: winit::event::DeviceEvent,
    ) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            self.input_state.borrow_mut().register_mouse_delta(delta);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        // Propagate all windows events to imgui
        let copied_event = event.clone();
        let generic_event = winit::event::Event::<()>::WindowEvent {
            window_id,
            event: copied_event,
        };
        self.winit_platform
            .handle_event(self.imgui_context.io_mut(), &self.window, &generic_event);

        match event {
            winit::event::WindowEvent::RedrawRequested => {
                // MAIN RENDER LOOP
                let start_render_loop = Instant::now();
                let scene = self
                    .active_scene
                    .as_mut()
                    .expect("Cannot render: No active scene");
                let dt = self
                    .current_frame_start
                    .duration_since(self.prev_frame_start)
                    .as_secs_f32();
                self.metrics.sma_dt.add(dt);

                // SCENE TICK
                let start_tick = Instant::now();
                scene.tick(dt);
                self.metrics.sma_tick_time.add_elapsed(start_tick);

                // SCENE RENDER
                let start_render = Instant::now();
                scene.render();
                self.metrics.sma_render_time.add_elapsed(start_render);

                // UI Renders
                let ui = self.imgui_context.frame();
                {
                    let title = scene.get_title();
                    let camera_rc = scene.get_main_camera();
                    let camera = camera_rc.borrow();
                    ui.window(format!("Scene: {title}"))
                        .size([300.0, 300.0], imgui::Condition::FirstUseEver)
                        .position([0.0, 0.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.text("Camera");
                            ui.text(format!(
                                "Position: ({:.1},{:.1},{:.1})",
                                camera.position.x, camera.position.y, camera.position.z,
                            ));
                        });
                }
                scene.render_ui(ui);
                self.metrics.render_ui(ui);

                // IMGUI Render logic
                self.winit_platform.prepare_render(ui, &self.window);
                let draw_data = self.imgui_context.render();
                self.ig_renderer
                    .render(draw_data)
                    .expect("error rendering imgui");
                let start_swap_time = Instant::now();
                self.surface
                    .swap_buffers(&self.glutin_context)
                    .expect("Failed to swap buffers");
                self.metrics.sma_swap_time.add_elapsed(start_swap_time);

                // Automatic scene swap
                if self.max_scene_duration_secs > 0.0
                    && self
                        .current_frame_start
                        .duration_since(self.active_scene_started_at.unwrap())
                        .as_secs_f32()
                        > self.max_scene_duration_secs
                {
                    info!("Maximum scene time reached. Collecting scene stats");
                    let benchmark_output_path = format!(
                        "output/benchmark_{}.csv",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .expect("Time goes forward")
                            .as_secs_f32()
                    );

                    let stats = scene.get_stats();
                    stats.print_scene_stats();
                    stats
                        .save_scene_stats(&benchmark_output_path)
                        .expect("Unable to write scene stats");
                    if self.available_scenes.is_empty() {
                        info!(
                            "No more scenes left. Results can be found at {benchmark_output_path}"
                        );
                        event_loop.exit();
                    } else {
                        self.start_next_scene().expect("Could not start next scene");
                    }
                }
                self.metrics.sma_render_loop.add_elapsed(start_render_loop);
            }
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            winit::event::WindowEvent::MouseInput {
                device_id: _device_id,
                state,
                button,
            } => match state {
                winit::event::ElementState::Pressed => {
                    self.input_state.borrow_mut().mouse_button_pressed(button);
                }
                winit::event::ElementState::Released => {
                    self.input_state.borrow_mut().mouse_button_released(&button);
                }
            },
            winit::event::WindowEvent::KeyboardInput {
                device_id: _device_id,
                event,
                is_synthetic: _is_synthetic,
            } => match event.physical_key {
                winit::keyboard::PhysicalKey::Code(code) => {
                    // Exit program when esc pressed
                    if code == KeyCode::Escape {
                        error!("User hit ESCAPE. Exiting program");
                        event_loop.exit();
                    }
                    match event.state {
                        winit::event::ElementState::Pressed => {
                            self.input_state.borrow_mut().key_pressed(code)
                        }
                        winit::event::ElementState::Released => {
                            self.input_state.borrow_mut().key_released(&code)
                        }
                    };
                }
                winit::keyboard::PhysicalKey::Unidentified(_c) => {
                    error!("Unknwown key pressed");
                }
            },
            winit::event::WindowEvent::Resized(new_size) => {
                if new_size.width > 0 && new_size.height > 0 {
                    self.surface.resize(
                        &self.glutin_context,
                        NonZeroU32::new(new_size.width).unwrap(),
                        NonZeroU32::new(new_size.height).unwrap(),
                    );
                }
            }
            _ => {}
        }
    }
}

impl Application {
    pub fn new(title: &str) -> Result<Application, Box<dyn Error>> {
        let frame_width = 1920;
        let frame_height = 1080;

        // Common setup for creating a winit window and imgui context, not specifc
        // to this renderer at all except that glutin is used to create the window
        // since it will give us access to a GL context
        let (event_loop, window, surface, context) =
            create_window(title, frame_width, frame_height);
        let (winit_platform, mut imgui_context) = imgui_init(&window);

        // OpenGL context from glow
        let gl = glow_context(&context);

        // OpenGL renderer from this crate
        let ig_renderer = imgui_glow_renderer::AutoRenderer::new(gl, &mut imgui_context)?;
        Ok(Self {
            active_scene: None,
            active_scene_started_at: None,
            available_scenes: VecDeque::new(),
            current_frame_start: Instant::now(),
            event_loop: Some(event_loop),
            glutin_context: context,
            ig_renderer,
            metrics: SceneMetrics::new(),
            imgui_context,
            input_state: Rc::new(RefCell::new(InputState::new())),
            max_scene_duration_secs: 0.0,
            prev_frame_start: Instant::now(),
            surface,
            window,
            winit_platform,
        })
    }

    pub fn gl_context(&self) -> &Rc<glow::Context> {
        self.ig_renderer.gl_context()
    }

    pub fn add_scene(&mut self, scene: Box<dyn Scene>) {
        self.available_scenes.push_back(scene);
    }

    fn start_next_scene(&mut self) -> Result<(), Box<dyn Error>> {
        let mut next_scene = self
            .available_scenes
            .pop_front()
            .ok_or(std::io::Error::other(
                "No more scenes available. Did you forget to add them?",
            ))?;
        next_scene.start();
        self.active_scene = Some(next_scene);
        self.active_scene_started_at = Some(Instant::now());
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.start_next_scene()?;

        // Start event loop
        let event_loop = self
            .event_loop
            .take()
            .ok_or(std::io::Error::other("Could not fetch event loop"))?;
        event_loop.run_app(self)?;
        Ok(())
    }
}

fn create_window(
    title: &str,
    width: u32,
    height: u32,
) -> (
    EventLoop<()>,
    Window,
    Surface<WindowSurface>,
    PossiblyCurrentContext,
) {
    let event_loop = EventLoop::new().unwrap();

    let window_attributes = WindowAttributes::default()
        .with_title(title)
        // .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .with_inner_size(LogicalSize::new(1920, 1080));
    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_attributes(Some(window_attributes))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .expect("Failed to create OpenGL window");

    let window = window.unwrap();
    window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .expect("Failed to grab cursor");

    let context_attribs =
        ContextAttributesBuilder::new().build(Some(window.window_handle().unwrap().as_raw()));
    let context = unsafe {
        cfg.display()
            .create_context(&cfg, &context_attribs)
            .expect("Failed to create OpenGL context")
    };

    let surface_attribs = SurfaceAttributesBuilder::<WindowSurface>::new()
        .with_srgb(Some(true))
        .build(
            window.window_handle().unwrap().as_raw(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
    let surface = unsafe {
        cfg.display()
            .create_window_surface(&cfg, &surface_attribs)
            .expect("Failed to create OpenGL surface")
    };
    let context = context
        .make_current(&surface)
        .expect("Failed to make OpenGL context current");

    if !USE_VSYNC {
        info!("Disabling VSYNC");
        surface
            .set_swap_interval(&context, SwapInterval::DontWait)
            .expect("Unable to disable vsync");
    }

    (event_loop, window, surface, context)
}

fn glow_context(context: &PossiblyCurrentContext) -> glow::Context {
    unsafe {
        glow::Context::from_loader_function_cstr(|s| context.display().get_proc_address(s).cast())
    }
}

fn imgui_init(window: &Window) -> (WinitPlatform, imgui::Context) {
    let mut imgui_context = imgui::Context::create();
    imgui_context.set_ini_filename(None);

    let mut winit_platform = WinitPlatform::new(&mut imgui_context);
    winit_platform.attach_window(
        imgui_context.io_mut(),
        window,
        imgui_winit_support::HiDpiMode::Rounded,
    );

    imgui_context
        .fonts()
        .add_font(&[imgui::FontSource::DefaultFontData { config: None }]);

    imgui_context.io_mut().font_global_scale = (1.0 / winit_platform.hidpi_factor()) as f32;

    (winit_platform, imgui_context)
}
