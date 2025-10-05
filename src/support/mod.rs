// This file is based on code from imgui-rs, originally licensed under MIT
// Modifications (c) 2025 8Oldr, distributed under GPLv3

use glium::glutin::config::ConfigTemplateBuilder;
use glium::glutin::context::ContextAttributesBuilder;
use glium::glutin::display::GetGlDisplay;
use glium::glutin::prelude::{GlDisplay, NotCurrentGlContext};
use glium::glutin::surface::{GlSurface, SurfaceAttributesBuilder, WindowSurface};
use glium::winit::event::DeviceId;
use glium::winit::keyboard::Key;
use glium::winit::raw_window_handle::HasWindowHandle;
use glium::{Display, Surface};
use imgui::{Context, FontConfig, FontGlyphRanges, FontSource, Ui};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::winit::dpi::LogicalSize;
use imgui_winit_support::winit::event::{Event, WindowEvent};
use imgui_winit_support::winit::event_loop::EventLoop;
use imgui_winit_support::winit::window::WindowAttributes;
use imgui_winit_support::{HiDpiMode, WinitPlatform};
use std::num::NonZeroU32;
use std::path::Path;
use std::time::Instant;

mod clipboard;

pub const FONT_SIZE: f32 = 13.0;

#[allow(dead_code)] // annoyingly, RA yells that this is unusued
pub fn simple_init<F: FnMut(&mut bool, &mut Ui, &mut Renderer, &Option<Key>) + 'static>(
    title: &str,
    run_ui: F,
) {
    init_with_startup(title, |_, _, _| {}, run_ui);
}

pub fn init_with_startup<FInit, FUi>(title: &str, mut startup: FInit, mut run_ui: FUi)
where
    FInit: FnMut(&mut Context, &mut Renderer, &Display<WindowSurface>) + 'static,
    FUi: FnMut(&mut bool, &mut Ui, &mut Renderer, &Option<Key>) + 'static,
{
    let mut imgui = create_context();

    let title = match Path::new(&title).file_name() {
        Some(file_name) => file_name.to_str().unwrap(),
        None => title,
    };
    let event_loop = EventLoop::new().expect("Failed to create EventLoop");

    let window_attributes = WindowAttributes::default()
        .with_title(title)
        .with_inner_size(LogicalSize::new(1024, 768));

    let config_template = ConfigTemplateBuilder::new();
    let display_builder =
        glutin_winit::DisplayBuilder::new().with_window_attributes(Some(window_attributes));

    let (window, gl_config) = display_builder
        .build(&event_loop, config_template, |mut configs| {
            configs.next().unwrap()
        })
        .unwrap();

    let window = window.unwrap(); // Safe, we requested Some(window_attributes)

    let context_attributes =
        ContextAttributesBuilder::new().build(Some(window.window_handle().unwrap().as_raw()));

    let not_current_gl_context = unsafe {
        gl_config
            .display()
            .create_context(&gl_config, &context_attributes)
            .unwrap()
    };

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        window.window_handle().unwrap().as_raw(),
        NonZeroU32::new(1024).unwrap(),
        NonZeroU32::new(768).unwrap(),
    );

    let surface = unsafe {
        gl_config
            .display()
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };
    let gl_context = not_current_gl_context.make_current(&surface).unwrap();

    surface
        .set_swap_interval(
            &gl_context,
            glium::glutin::surface::SwapInterval::Wait(NonZeroU32::new(1).unwrap()),
        )
        .unwrap();

    let display = glium::Display::from_context_surface(gl_context, surface).unwrap();

    let mut renderer = Renderer::new(&mut imgui, &display).expect("Failed to initialize renderer");

    if let Some(backend) = clipboard::init() {
        imgui.set_clipboard_backend(backend);
    } else {
        eprintln!("Failed to initialize clipboard");
    }

    let mut platform = WinitPlatform::new(&mut imgui);
    {
        let dpi_mode = if let Ok(factor) = std::env::var("IMGUI_EXAMPLE_FORCE_DPI_FACTOR") {
            // Allow forcing of HiDPI factor for debugging purposes
            match factor.parse::<f64>() {
                Ok(f) => HiDpiMode::Locked(f),
                Err(e) => panic!("Invalid scaling factor: {}", e),
            }
        } else {
            HiDpiMode::Default
        };

        platform.attach_window(imgui.io_mut(), &window, dpi_mode);
    }

    let mut last_frame = Instant::now();

    startup(&mut imgui, &mut renderer, &display);

    let mut key_pressed: Option<Key> = None;

    #[allow(deprecated)]
    event_loop
        .run(move |event, window_target| {
            platform.handle_event(imgui.io_mut(), &window, &event);
            match event {
                Event::NewEvents(_) => {
                    let now = Instant::now();
                    imgui.io_mut().update_delta_time(now - last_frame);
                    last_frame = now;
                }
                Event::AboutToWait => {
                    platform
                        .prepare_frame(imgui.io_mut(), &window)
                        .expect("Failed to prepare frame");
                    window.request_redraw();
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let ui = imgui.frame();

                    let mut run = true;
                    run_ui(&mut run, ui, &mut renderer, &key_pressed);
                    if !run {
                        window_target.exit();
                    }

                    let mut target = display.draw();
                    target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
                    platform.prepare_render(ui, &window);
                    let draw_data = imgui.render();
                    renderer
                        .render(&mut target, draw_data)
                        .expect("Rendering failed");
                    target.finish().expect("Failed to swap buffers");
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(new_size),
                    ..
                } => {
                    if new_size.width > 0 && new_size.height > 0 {
                        display.resize((new_size.width, new_size.height));
                    }
                    platform.handle_event(imgui.io_mut(), &window, &event);
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => window_target.exit(),
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => {
                    let key = event.logical_key.clone();
                    match event.state {
                        glium::winit::event::ElementState::Pressed => key_pressed = Some(key),
                        glium::winit::event::ElementState::Released => key_pressed = None,
                    }
                }
                event => {
                    platform.handle_event(imgui.io_mut(), &window, &event);
                }
            }
        })
        .expect("EventLoop error");
}

/// Creates the imgui context
pub fn create_context() -> imgui::Context {
    let mut imgui = Context::create();
    // Fixed font size. Note imgui_winit_support uses "logical
    // pixels", which are physical pixels scaled by the devices
    // scaling factor. Meaning, 13.0 pixels should look the same size
    // on two different screens, and thus we do not need to scale this
    // value (as the scaling is handled by winit)
    imgui.fonts().add_font(&[
        FontSource::TtfData {
            data: include_bytes!("../../resources/Roboto-Regular.ttf"),
            size_pixels: FONT_SIZE,
            config: Some(FontConfig {
                // As imgui-glium-renderer isn't gamma-correct with
                // it's font rendering, we apply an arbitrary
                // multiplier to make the font a bit "heavier". With
                // default imgui-glow-renderer this is unnecessary.
                rasterizer_multiply: 1.5,
                // Oversampling font helps improve text rendering at
                // expense of larger font atlas texture.
                oversample_h: 4,
                oversample_v: 4,
                ..FontConfig::default()
            }),
        },
        FontSource::TtfData {
            data: include_bytes!("../../resources/mplus-1p-regular.ttf"),
            size_pixels: FONT_SIZE,
            config: Some(FontConfig {
                // Oversampling font helps improve text rendering at
                // expense of larger font atlas texture.
                oversample_h: 4,
                oversample_v: 4,
                // Range of glyphs to rasterize
                glyph_ranges: FontGlyphRanges::japanese(),
                ..FontConfig::default()
            }),
        },
    ]);
    imgui.set_ini_filename(None);

    imgui
}
