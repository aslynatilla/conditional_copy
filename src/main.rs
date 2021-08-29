use glium::glutin;
use glium::glutin::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{Display, Surface};
use imgui::*;
use imgui::{Context, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use std::env;
use std::fs;
use std::time::Instant;

mod cc_controller;
use cc_controller::Controller;

const WIDTH: i32 = 600;
const HEIGHT: i32 = 800;

fn retrieve_instructions(path: &String) -> String {
    match fs::read_to_string(&path) {
        Ok(file_content) => file_content,
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => panic!("The path to the specified instruction file was not correct. The file was not found."),
            _ => panic!("Unexpected error in reading the file.")
        }
    }
}

fn main() {
    let path = match env::args().nth(1) {
        Some(s) => s,
        None => String::from("cc_instructions.cci"),
    };

    let instructions = retrieve_instructions(&path);
    let controller = Controller::new(instructions);
    let targets = controller.target_list();

    let filename: String = std::path::Path::new(&path)
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let event_loop = EventLoop::new();
    let context_builder = glutin::ContextBuilder::new().with_vsync(true);
    let window_builder = WindowBuilder::new()
        .with_title("Conditional Copy Script")
        .with_inner_size(glutin::dpi::LogicalSize::new(WIDTH, HEIGHT))
        .with_resizable(false);
    let display = Display::new(window_builder, context_builder, &event_loop)
        .expect("Failed to initialize glium Display");

    let mut imgui = Context::create();
    imgui.set_ini_filename(None);

    let mut platform = WinitPlatform::init(&mut imgui);
    {
        let gl_window = display.gl_window();
        let window = gl_window.window();
        //  this line is why we need glium - "0.29.0"
        //  TODO: find a way to use 0.30.0 (?)
        platform.attach_window(imgui.io_mut(), window, HiDpiMode::Rounded);
    }

    let hidpi_factor = platform.hidpi_factor();
    let font_size = (26.0 * hidpi_factor) as f32;

    imgui.fonts().add_font(&[FontSource::TtfData {
        data: include_bytes!("../resources/Roboto-Light.ttf"),
        size_pixels: font_size,
        config: None,
    }]);

    imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

    let imgui_style = imgui.style_mut();
    imgui_style.use_light_colors();

    let mut renderer =
        Renderer::init(&mut imgui, &display).expect("Failed to initialize imgui-glium Renderer.");

    let mut last_frame = Instant::now();

    event_loop.run(move |event, _, control_flow| match event {
        Event::NewEvents(_) => {
            let now = Instant::now();
            imgui.io_mut().update_delta_time(now - last_frame);
            last_frame = now;
        }
        Event::MainEventsCleared => {
            let gl_window = display.gl_window();
            platform
                .prepare_frame(imgui.io_mut(), gl_window.window())
                .expect("Failed to prepare frame");
            gl_window.window().request_redraw();
        }
        Event::RedrawRequested(_) => {
            let ui = imgui.frame();

            // let mut run = !ui.is_any_item_focused() && !ui.is_key_down(Key::Escape);
            let mut run = true;

            Window::new(im_str!("Conditional Copy Window"))
                .opened(&mut run)
                .size([WIDTH as f32, HEIGHT as f32], Condition::Always)
                .position([0.0, 0.0], Condition::Appearing)
                .movable(false)
                .resizable(false)
                .no_decoration()
                .build(&ui, || {
                    let title = ImString::new(format!("Instructions in {}", filename));
                    ui.text(title);
                    let available_dims = ui.content_region_avail();
                    ChildWindow::new("InstructionWindow")
                        .size(available_dims)
                        .scrollable(true)
                        .border(true)
                        .build(&ui, || {
                            for files_to_read in targets.iter() {
                                ui.text_colored([0.6, 0.0, 0.0, 1.0], format!("{}", files_to_read));
                            }
                        });
                });

            if !run {
                *control_flow = ControlFlow::Exit;
            }

            let gl_window = display.gl_window();
            let mut target = display.draw();
            target.clear_color_srgb(1.0, 1.0, 1.0, 1.0);
            platform.prepare_render(&ui, gl_window.window());
            let draw_data = ui.render();
            renderer
                .render(&mut target, draw_data)
                .expect("Rendering failed");
            target.finish().expect("Failed to swap buffers");
        }
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        },

        _ => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
        }
    })
}
