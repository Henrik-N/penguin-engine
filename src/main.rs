#![allow(unused)]
//use std::task::ready;
use anyhow::*;

mod core;
mod ecs;
mod engine;

use ash::vk;

use legion::world::SubWorld;
use legion::*;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode};
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};


use std::time::{Duration, Instant};
use std::vec::from_elem;
use crate::core::config;
use crate::engine::Renderer;



pub struct App {
    pub world: World,
    pub resources: Resources,

    pub startup_schedule: Schedule,
    pub run_schedule: Schedule,
    pub shutdown_schedule: Schedule,
}
impl App {
    fn run(mut self, event_loop: EventLoop<()>) -> Result<()> {

        self.startup_schedule.execute(&mut self.world, &mut self.resources);

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
                            },
                            _ => {
                                let input_events_resource = self.resources
                                    .get_mut::<crate::core::input::InputEvents>();
                                input_events_resource.unwrap().update(input);
                            }
                        }
                    },
                    _ => {
                    },
                },
                // Other events ------------
                Event::MainEventsCleared => {
                    self.run_schedule.execute(&mut self.world, &mut self.resources)
                },
                Event::RedrawRequested(_window_id) => {},
                Event::LoopDestroyed => {
                    self.shutdown_schedule.execute(&mut self.world, &mut self.resources);
                }
                _ => (),
            }
        });
    }
}


pub struct RendResource {
    renderer: Renderer,
}
impl RendResource {
    fn new(window: &winit::window::Window) -> Self {
        let mut renderer = crate::engine::Renderer::create(&window).expect("Couldn't create game!");

        Self {
            renderer,
        }
    }

    fn draw(&mut self, delta_time: f32) {
        self.renderer.draw(delta_time);
    }
}

use std::collections::HashSet;
use crate::core::time::PTime;
use crate::ecs::{AppBuilder};


#[system]
fn render_system(#[resource] rend: &mut RendResource, #[resource] time: &PTime) {
    rend.draw(time.delta());
}

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

    let app = AppBuilder::builder()
        .add_plugin::<crate::core::time::TimePlugin>()
        .add_plugin::<crate::core::input::InputPlugin>()
        .insert_resource(RendResource::new(&window))
        .add_run_steps(Schedule::builder()
            .add_thread_local(render_system_system())
            .build().into_vec())
        .build();

    app.run(event_loop)?;

    Ok(())
}
