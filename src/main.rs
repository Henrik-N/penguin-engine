#![allow(unused)]
use anyhow::*;

mod core;
mod ecs;
mod engine;

use legion::*;
use winit::event::WindowEvent;
use winit::event::{ElementState, Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};


use crate::core::config;



pub struct App {
    pub world: World,
    pub resources: Resources,

    pub startup_schedule: Schedule,
    pub run_schedule: Schedule,
    pub shutdown_schedule: Schedule,
}
impl App {
    fn run(mut self, event_loop: EventLoop<()>) -> Result<()> {
        self.startup_schedule
            .execute(&mut self.world, &mut self.resources);

        //return Ok(());

        event_loop.run(move |event, _, control_flow| {
            // game_instance.render(1_f32);
            *control_flow = ControlFlow::Poll;

            match event {
                // Window events ------------
                Event::WindowEvent { event, .. } => match event {
                    WindowEvent::CloseRequested => {
                        log::info!("Close requested, closing...");
                        *control_flow = ControlFlow::Exit;
                    }

                    //    } => match (virtual_keycode, state) {
                    //        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                    //            *control_flow = ControlFlow::Exit
                    //        }
                    WindowEvent::KeyboardInput { input, .. } => {
                        match (input.virtual_keycode, input.state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit
                            }
                            _ => {
                                let input_events_resource =
                                    self.resources.get_mut::<crate::core::input::InputEvents>();
                                input_events_resource.unwrap().update(input);
                            }
                        }
                    }
                    _ => {}
                },
                // Other events ------------
                Event::MainEventsCleared => {
                    self.run_schedule
                        .execute(&mut self.world, &mut self.resources);
                }
                Event::RedrawRequested(_window_id) => {}
                Event::LoopDestroyed => {
                    //println!("reached here");
                    self.shutdown_schedule
                        .execute(&mut self.world, &mut self.resources);
                }
                _ => (),
            }
        });
    }
}



use crate::ecs::AppBuilder;

pub fn create_window(
    event_loop: &winit::event_loop::EventLoop<()>,
    width: u32,
    height: u32,
) -> winit::window::Window {
    winit::window::WindowBuilder::new()
        .with_title("penguin engine")
        .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
        .build(&event_loop)
        .expect("Window could not be created")
}

fn main() -> Result<()> {
    crate::core::logger::init_logger().expect("Couldn't init logger");
    log::trace!("Logger initialized");

    let event_loop = winit::event_loop::EventLoop::new();

    let window = create_window(&event_loop, config::WIDTH, config::HEIGHT);
    log::trace!("Window created");

    //let renderer = Renderer::create(&window);

    //return Ok(());

    let app = AppBuilder::builder()
        .add_plugin(crate::core::time::TimePlugin)
        .add_plugin(crate::core::input::InputPlugin)
        .add_plugin(crate::engine::renderer::RendererPlugin {
            window: Some(window),
        })
        //.insert_resource(RendResource::new(&window))
        //.add_run_steps(Schedule::builder()
        //    .add_thread_local(render_system_system())
        //    .build().into_vec())
        .build();

    app.run(event_loop)?;

    Ok(())
}
