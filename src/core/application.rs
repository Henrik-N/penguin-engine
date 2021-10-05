use crate::core::time::PTime;
use std::time::Instant;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};

use anyhow::Result;
use crate::engine::PenguinEngine;
use super::config;

use crate::core::{logger, events};

/// Settings for the window when first starting the application

pub struct Application {
    pub(crate) ptime: PTime,
    pub window: winit::window::Window,
}


impl Application {
    pub fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        super::logger::init_logger();
        log::trace!("Logger initialized");

        let window = Self::create_window(&event_loop, config::WIDTH, config::HEIGHT);
        log::trace!("Window created");

        let ptime = PTime::create_system();
        log::trace!("Time system created");



        Self {
            ptime,
            window,
        }
    }

    pub fn create_window(event_loop: &winit::event_loop::EventLoop<()>, width: u32, height: u32) -> winit::window::Window {
        winit::window::WindowBuilder::new()
            .with_title("penguin engine")
            .with_inner_size(winit::dpi::PhysicalSize::new(width, height))
            .build(&event_loop)
            .expect("Window could not be created")
    }

    pub(crate) fn init() {
        logger::init_logger();
    }

    pub(crate) fn run(mut self, event_loop: EventLoop<()>) -> Result<()> {
        let mut game_instance = PenguinEngine::create(&self.window)?;



        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                // Window events ------------
                | Event::WindowEvent { event, .. } => match event {
                    | WindowEvent::CloseRequested => {
                        log::info!("Close requested, closing...");
                        *control_flow = ControlFlow::Exit;
                    }
                    | WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            virtual_keycode,
                            state,
                            ..
                        } => match (virtual_keycode, state) {
                            (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                                *control_flow = ControlFlow::Exit
                            }
                            _ => Self::process_keyboard_input(virtual_keycode, state),
                        },
                    },
                    _ => {}
                },
                // Other events ------------
                // render next frame if app is not being destroyed
                | Event::MainEventsCleared => {
                    self.ptime.tick();

                    game_instance.update(self.ptime.resource().delta());

                    self.window.request_redraw();
                },
                | Event::RedrawRequested(_window_id) => {
                    game_instance.render(self.ptime.resource().delta());
                },
                | Event::LoopDestroyed => {
                    game_instance.shutdown();
                },
                _ => (),
            }
        });
    }

    fn process_keyboard_input(virtual_keycode: Option<VirtualKeyCode>, state: ElementState) {
        match (virtual_keycode, state) {
            (Some(VirtualKeyCode::A), ElementState::Pressed) => {
                println!("Pressing A");
            }
            _ => {}
        }
    }
}
