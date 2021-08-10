// -------------------------- WINDOW ----------------------------
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

pub fn init_window(event_loop: &EventLoop<()>) -> Window {
    WindowBuilder::new()
        .with_title("penguin engine")
        .with_inner_size(winit::dpi::PhysicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap()
}
