use glium::glutin;
use glium::glutin::event::{Event, WindowEvent};
use glium::glutin::event_loop::{ControlFlow, EventLoop};
use glium::glutin::window::WindowBuilder;
use glium::{Display, Surface};
use imgui::*;
use imgui::{Context, FontSource};
use imgui_glium_renderer::Renderer;
use imgui_winit_support::{HiDpiMode, WinitPlatform};

use std::time::Instant;

//https://github.com/imgui-rs/imgui-rs

const WIDTH: i32 = 600;
const HEIGHT: i32 = 800;

fn main() {
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

            let mut run = !ui.is_any_item_focused() && !ui.is_key_down(Key::Escape);

            Window::new(im_str!("Creating a long window"))
                .opened(&mut run)
                .size([WIDTH as f32, HEIGHT as f32], Condition::Always)
                .position([0.0, 0.0], Condition::Appearing)
                .movable(false)
                .resizable(false)
                .no_decoration()
                .build(&ui, || {
                    ui.text(im_str!("Hello world!"));
                    ui.checkbox(im_str!("One"), &mut false);
                    ui.checkbox(im_str!("Two"), &mut true);

                    TreeNode::new(im_str!("TreeNode #1")).build(&ui, || {
                        ui.bullet_text(im_str!("Test Text"));
                        ui.checkbox(im_str!("Hello"), &mut true);
                    });

                    ChildWindow::new("whatever")
                        .size([300.0, 200.0])
                        .scrollable(true)
                        .border(true)
                        .build(&ui, || {
                            for i in 1..=4 {
                                ui.text_colored(
                                    [i as f32 * 0.2, 0.0, 0.0, 1.0],
                                    format!("Hello text #{}", i),
                                );
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
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => *control_flow = ControlFlow::Exit,
        event => {
            let gl_window = display.gl_window();
            platform.handle_event(imgui.io_mut(), gl_window.window(), &event);
        }
    })
}
