mod core;
mod ecs;
mod engine;

use anyhow::*;
use crate::core::application::Application;


fn main() -> Result<()> {
    // request game instance from app
    let event_loop = winit::event_loop::EventLoop::new();

    let app = Application::new(&event_loop);
    app.run(event_loop)?;

    Ok(())
}
