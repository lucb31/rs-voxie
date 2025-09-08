// Derived from https://github.com/imgui-rs/imgui-glow-renderer/blob/main/examples/glow_01_basic.rs
use std::{collections::HashSet, num::NonZeroU32, time::Instant};

use glutin::{
    config::ConfigTemplateBuilder,
    context::{ContextAttributesBuilder, NotCurrentGlContext, PossiblyCurrentContext},
    display::{GetGlDisplay, GlDisplay},
    surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface},
};
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

mod camera;
mod cube;
mod renderer;
mod triangle;

const TITLE: &str = "Rustcraft";

fn main() {
    // Common setup for creating a winit window and imgui context, not specifc
    // to this renderer at all except that glutin is used to create the window
    // since it will give us access to a GL context
    let (event_loop, window, surface, context) = create_window();
    let (mut winit_platform, mut imgui_context) = imgui_init(&window);

    // OpenGL context from glow
    let gl = glow_context(&context);

    // OpenGL renderer from this crate
    let mut ig_renderer = imgui_glow_renderer::AutoRenderer::new(gl, &mut imgui_context)
        .expect("failed to create renderer");

    let mut keys_pressed = HashSet::<KeyCode>::new();
    let mut mouse_buttons_pressed = HashSet::<MouseButton>::new();
    let mut game_renderer = renderer::Renderer::new(ig_renderer.gl_context());

    let mut last_frame = Instant::now();

    // Standard winit event loop
    #[allow(deprecated)]
    event_loop
        .run(move |event, window_target| {
            match event {
                winit::event::Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui_context
                        .io_mut()
                        .update_delta_time(now.duration_since(last_frame));
                    last_frame = now;
                }
                winit::event::Event::AboutToWait => {
                    winit_platform
                        .prepare_frame(imgui_context.io_mut(), &window)
                        .unwrap();
                    window.request_redraw();
                }
                // Propagate mouse movement to camera
                winit::event::Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta },
                    ..
                } => {
                    if mouse_buttons_pressed.contains(&MouseButton::Middle) {
                        game_renderer
                            .camera
                            .process_mouse_movement(delta.0, delta.1);
                    }
                }
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::RedrawRequested,
                    ..
                } => {
                    // MAIN RENDER LOOP
                    let ctx = ig_renderer.gl_context();

                    // Update camera position based on inputs
                    let dt = last_frame.elapsed().as_secs_f32();
                    for key in &keys_pressed {
                        game_renderer.camera.process_keyboard(*key, dt);
                    }
                    game_renderer.render(ctx);

                    let ui = imgui_context.frame();
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
                                game_renderer.camera.position.x,
                                game_renderer.camera.position.y,
                                game_renderer.camera.position.z,
                            ));
                            ui.text(format!(
                                "pitch, yaw: ({:.3},{:.3})",
                                game_renderer.camera.pitch, game_renderer.camera.yaw,
                            ));
                            ui.slider("Speed", 50.0, 5000.0, &mut game_renderer.camera.speed);
                            ui.slider(
                                "Sensitivity",
                                0.001,
                                0.01,
                                &mut game_renderer.camera.sensitivity,
                            )
                        });

                    winit_platform.prepare_render(ui, &window);
                    let draw_data = imgui_context.render();

                    // This is the only extra render step to add
                    ig_renderer
                        .render(draw_data)
                        .expect("error rendering imgui");

                    surface
                        .swap_buffers(&context)
                        .expect("Failed to swap buffers");
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
                            device_id,
                            state,
                            button,
                        },
                    ..
                } => {
                    match state {
                        winit::event::ElementState::Pressed => mouse_buttons_pressed.insert(button),
                        winit::event::ElementState::Released => {
                            mouse_buttons_pressed.remove(&button)
                        }
                    };
                }
                winit::event::Event::WindowEvent {
                    event:
                        winit::event::WindowEvent::KeyboardInput {
                            device_id,
                            event,
                            is_synthetic,
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
                            winit::event::ElementState::Pressed => keys_pressed.insert(code),
                            winit::event::ElementState::Released => keys_pressed.remove(&code),
                        };
                    }
                    winit::keyboard::PhysicalKey::Unidentified(c) => {
                        println!("Unknwown key pressed");
                    }
                },
                winit::event::Event::WindowEvent {
                    event: winit::event::WindowEvent::Resized(new_size),
                    ..
                } => {
                    if new_size.width > 0 && new_size.height > 0 {
                        surface.resize(
                            &context,
                            NonZeroU32::new(new_size.width).unwrap(),
                            NonZeroU32::new(new_size.height).unwrap(),
                        );
                    }
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }
                winit::event::Event::LoopExiting => {
                    let gl = ig_renderer.gl_context();
                    game_renderer.destroy(gl);
                }
                event => {
                    winit_platform.handle_event(imgui_context.io_mut(), &window, &event);
                }
            }
        })
        .expect("EventLoop error");
}

fn create_window() -> (
    EventLoop<()>,
    Window,
    Surface<WindowSurface>,
    PossiblyCurrentContext,
) {
    let event_loop = EventLoop::new().unwrap();

    let window_attributes = WindowAttributes::default()
        .with_title(TITLE)
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
            NonZeroU32::new(1024).unwrap(),
            NonZeroU32::new(768).unwrap(),
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
