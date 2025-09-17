// Derived from https://github.com/imgui-rs/imgui-glow-renderer/blob/main/examples/glow_01_basic.rs
use std::{
    collections::HashSet,
    error::Error,
    num::NonZeroU32,
    rc::Rc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
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

use crate::{camera, scene::Scene};

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

        // Used to limit render time per scene
        let mut first_frame_scene = Instant::now();
        let mut last_frame = Instant::now();
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
                    self.imgui_context
                        .io_mut()
                        .update_delta_time(now.duration_since(last_frame));
                    last_frame = now;
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
                    // MAIN RENDER LOOP
                    let ctx = self.gl_context().clone();

                    // Update camera position based on inputs
                    let dt = last_frame.elapsed().as_secs_f32();
                    for key in &self.keys_pressed {
                        scene.get_main_camera().process_keyboard(*key, dt);
                    }
                    // FIX: Ideally, this should be framerate independent.
                    // Dont know how to de-couple right now
                    scene.tick(dt);
                    scene.render(&ctx);
                    let camera = scene.get_main_camera();

                    let ui = self.imgui_context.frame();
                    ui.window("Camera Debug")
                        .size([300.0, 200.0], imgui::Condition::FirstUseEver)
                        .build(|| {
                            let mouse_pos = ui.io().mouse_pos;
                            ui.text(format!(
                                "Mouse Position: ({:.1},{:.1})",
                                mouse_pos[0], mouse_pos[1]
                            ));
                            ui.separator();
                            ui.text("Camera");
                            ui.text(format!(
                                "Position: ({:.3},{:.3},{:.3})",
                                camera.position.x, camera.position.y, camera.position.z,
                            ));
                            //                             ui.slider("Speed", 50.0, 5000.0, &mut scene.get_main_camera().speed);
                            //                             ui.slider(
                            //                                 "Sensitivity",
                            //                                 0.001,
                            //                                 0.01,
                            //                                 &mut scene.get_main_camera().sensitivity,
                            //                             )
                        });

                    self.winit_platform.prepare_render(ui, &self.window);
                    let draw_data = self.imgui_context.render();

                    // This is the only extra render step to add
                    self.ig_renderer
                        .render(draw_data)
                        .expect("error rendering imgui");

                    self.surface
                        .swap_buffers(&self.glutin_context)
                        .expect("Failed to swap buffers");

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
