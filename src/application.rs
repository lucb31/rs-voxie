// Derived from https://github.com/imgui-rs/imgui-glow-renderer/blob/main/examples/glow_01_basic.rs
use std::{
    collections::HashSet,
    error::Error,
    fmt::format,
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
use raw_window_handle::HasWindowHandle;
use winit::{
    event::{DeviceEvent, MouseButton},
    keyboard::KeyCode,
};

const USE_VSYNC: bool = true;

use crate::{scene::Scene, util::SimpleMovingAverage};

pub struct Application {
    pub max_scene_duration_secs: f32,
    // Input state
    keys_pressed: HashSet<KeyCode>,
    mouse_buttons_pressed: HashSet<MouseButton>,

    // Rendering & application loop context
    event_loop: Option<EventLoop<()>>,
    window: Window,
    surface: Surface<WindowSurface>,
    winit_platform: WinitPlatform,
    glutin_context: PossiblyCurrentContext,
    imgui_context: Context,
    ig_renderer: AutoRenderer,
}

impl Application {
    pub fn new(title: &str) -> Result<Application, Box<dyn Error>> {
        let keys_pressed = HashSet::<KeyCode>::new();
        let mouse_buttons_pressed = HashSet::<MouseButton>::new();
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
            glutin_context: context,
            winit_platform,
            imgui_context,
            max_scene_duration_secs: 0.0,
            keys_pressed,
            mouse_buttons_pressed,
            event_loop: Some(event_loop),
            window,
            surface,
            ig_renderer,
        })
    }

    pub fn gl_context(&self) -> &Rc<glow::Context> {
        self.ig_renderer.gl_context()
    }

    pub fn run(&mut self, mut scenes: Vec<Box<dyn Scene>>) -> Result<(), Box<dyn Error>> {
        if scenes.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No scenes provided",
            )));
        }
        let mut scene = scenes.pop().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No active scene found. Did you forget to set the scene?",
        ))?;
        let event_loop = self.event_loop.take().ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Could not fetch event loop",
        ))?;

        // Frame timings
        let mut first_frame_scene = Instant::now();
        let mut last_frame = Instant::now();
        let mut sma_dt = SimpleMovingAverage::new(100);
        let mut sma_render_time = SimpleMovingAverage::new(100);
        let mut sma_tick_time = SimpleMovingAverage::new(100);
        let mut sma_swap_time = SimpleMovingAverage::new(100);
        let mut sma_render_loop = SimpleMovingAverage::new(100);
        let mut dt: f32 = 0.0;
        scene.start();

        let benchmark_output_path = format!(
            "output/benchmark_{}.csv",
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs_f32()
        );

        // Standard winit event loop
        #[allow(deprecated)]
        event_loop.run(move |event, window_target| {
            match event {
                winit::event::Event::NewEvents(_) => {
                    let now = Instant::now();
                    let duration_since = now.duration_since(last_frame);
                    self.imgui_context
                        .io_mut()
                        .update_delta_time(duration_since);
                    last_frame = now;
                    dt = duration_since.as_secs_f32();
                }
                winit::event::Event::AboutToWait => {
                    self.winit_platform
                        .prepare_frame(self.imgui_context.io_mut(), &self.window)
                        .unwrap();
                    self.window.request_redraw();
                }
                // Propagate mouse movement to camera
                winit::event::Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    // FIX: This seems to conflict with imgui click events.
                    if self.mouse_buttons_pressed.contains(&MouseButton::Middle) {
                        scene
                            .get_main_camera()
                            .process_mouse_movement(delta.0, delta.1);
                    }
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::RedrawRequested,
                    ..
                } => {
                    let start_render_loop = Instant::now();
                    // MAIN RENDER LOOP
                    let ctx = self.gl_context().clone();

                    // Update camera position based on inputs
                    for key in &self.keys_pressed {
                        scene.get_main_camera().process_keyboard(*key);
                    }

                    // SCENE TICK
                    // FIX: Ideally, this should be framerate independent.
                    // Dont know how to de-couple right now
                    let start_tick = Instant::now();
                    scene.tick(dt, &ctx);
                    let tick_time_ns = start_tick.elapsed().as_secs_f32() * 1e6;
                    let avg_tick_time = sma_tick_time.add(tick_time_ns);

                    // SCENE RENDER
                    let start_render = Instant::now();
                    scene.render(&ctx);
                    let render_time_ns = start_render.elapsed().as_secs_f32() * 1e6;
                    let avg_render_time = sma_render_time.add(render_time_ns);
                    let avg_dt = sma_dt.add(dt);

                    let ui = self.imgui_context.frame();
                    let title = scene.get_title();
                    let camera = scene.get_main_camera();
                    ui.window(format!("Scene: {title}"))
                        .size([300.0, 200.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            ui.text("Camera");
                            ui.text(format!(
                                "Position: ({:.1},{:.1},{:.1})",
                                camera.position.x, camera.position.y, camera.position.z,
                            ));
                            ui.separator();
                            ui.text(format!("Avg FPS: {:.1}", 1.0 / avg_dt));
                            // Time physics simulation of the scene took
                            ui.text(format!("Scene: time to tick: {:.1} ns", avg_tick_time));
                            // Time it took to pass rendering logic and GPU command buffers
                            ui.text(format!("Scene: time to render: {:.1} ns", avg_render_time));
                            // Time it took to swap buffers. This is somehow representative of time
                            // that was spent waiting for the GPU (incl. any delay for VSync)
                            ui.text(format!("Swap time: {:.1} ns", sma_swap_time.get()));
                            ui.text(format!(
                                "Avg time per render loop: {:.1} ns",
                                sma_render_loop.get()
                            ));
                        });
                    scene.render_ui(ui);

                    self.winit_platform.prepare_render(ui, &self.window);
                    let draw_data = self.imgui_context.render();

                    // This is the only extra render step to add
                    self.ig_renderer
                        .render(draw_data)
                        .expect("error rendering imgui");

                    let start_swap_time = Instant::now();
                    self.surface
                        .swap_buffers(&self.glutin_context)
                        .expect("Failed to swap buffers");
                    sma_swap_time.add(start_swap_time.elapsed().as_secs_f32() * 1e6);

                    if self.max_scene_duration_secs > 0.0
                        && last_frame.duration_since(first_frame_scene).as_secs_f32()
                            > self.max_scene_duration_secs
                    {
                        println!("Maximum scene time reached. Collecting scene stats");
                        let stats = scene.get_stats();
                        stats.print_scene_stats();
                        stats
                            .save_scene_stats(&benchmark_output_path)
                            .expect("Unable to write scene stats");
                        if scenes.is_empty() {
                            println!("No more scenes left. Exiting...");
                            println!("Results can be found at {}", benchmark_output_path);
                            window_target.exit();
                        } else {
                            scene = scenes.pop().expect("Could not pop");
                            scene.start();
                            first_frame_scene = Instant::now();
                        }
                    }
                    sma_render_loop.add(start_render_loop.elapsed().as_secs_f32() * 1e6);
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::CloseRequested,
                    ..
                } => {
                    window_target.exit();
                }
                winit::event::Event::WindowEvent {
                    event:
                        winit::event::WindowEvent::MouseInput {
                            device_id: _device_id,
                            state,
                            button,
                        },
                    ..
                } => {
                    match state {
                        winit::event::ElementState::Pressed => {
                            self.mouse_buttons_pressed.insert(button)
                        }
                        winit::event::ElementState::Released => {
                            self.mouse_buttons_pressed.remove(&button)
                        }
                    };
                    self.winit_platform.handle_event(
                        self.imgui_context.io_mut(),
                        &self.window,
                        &event,
                    );
                }
                winit::event::Event::WindowEvent {
                    event:
                        winit::event::WindowEvent::KeyboardInput {
                            device_id: _device_id,
                            event,
                            is_synthetic: _is_synthetic,
                        },
                    ..
                } => match event.physical_key {
                    winit::keyboard::PhysicalKey::Code(code) => {
                        // Exit program when esc pressed
                        if code == KeyCode::Escape {
                            println!("User hit ESCAPE. Exiting program");
                            window_target.exit();
                        }
                        match event.state {
                            winit::event::ElementState::Pressed => self.keys_pressed.insert(code),
                            winit::event::ElementState::Released => self.keys_pressed.remove(&code),
                        };
                    }
                    winit::keyboard::PhysicalKey::Unidentified(_c) => {
                        println!("Unknwown key pressed");
                    }
                },
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(new_size),
                    ..
                } => {
                    if new_size.width > 0 && new_size.height > 0 {
                        self.surface.resize(
                            &self.glutin_context,
                            NonZeroU32::new(new_size.width).unwrap(),
                            NonZeroU32::new(new_size.height).unwrap(),
                        );
                    }
                    self.winit_platform.handle_event(
                        self.imgui_context.io_mut(),
                        &self.window,
                        &event,
                    );
                }
                winit::event::Event::LoopExiting => {
                    let gl = self.gl_context();
                    scene.destroy(gl);
                }
                event => {
                    self.winit_platform.handle_event(
                        self.imgui_context.io_mut(),
                        &self.window,
                        &event,
                    );
                }
            }
        })?;
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
        .with_inner_size(LogicalSize::new(1024, 768));
    let (window, cfg) = glutin_winit::DisplayBuilder::new()
        .with_window_attributes(Some(window_attributes))
        .build(&event_loop, ConfigTemplateBuilder::new(), |mut configs| {
            configs.next().unwrap()
        })
        .expect("Failed to create OpenGL window");

    let window = window.unwrap();

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
        println!("Disabling VSYNC");
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
