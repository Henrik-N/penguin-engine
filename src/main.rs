use anyhow::*;
use ash::vk;
use log::{debug, error, info, trace, warn};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::core::application::{Application};
use crate::core::time::PTimeResource;

// use engine::backend::pe::instance::create_ash_instance;

mod core;

mod ecs;
mod engine;



fn main() -> Result<()> {

    // request game instance from app
    let event_loop = winit::event_loop::EventLoop::new();

    let app = Application::new(&event_loop);
    app.run(event_loop)?;


    Ok(())





    //let shaders = pe::shaders::Shaders::init();

    // let mut engine = pe::PenguinEngine::init_engine(&window);
    // let device = &engine.device;
    //
    // let pipeline = device.build_graphics_pipeline().build();
    // pipeline.drop(&device);
}
