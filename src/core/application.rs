use crate::resources::time::PTime;
use std::time::Instant;
use winit::event_loop::ControlFlow;
use winit::event::{Event, WindowEvent, KeyboardInput, VirtualKeyCode, ElementState};

use anyhow::Result;
use crate::game_types::GameInstance;

use crate::core::logger;

/// Settings for the window when first starting the application
pub struct ApplicationConfig {
    pub(crate) start_x: u16,
    pub(crate) start_y: u16,
    pub(crate) start_width: u16,
    pub(crate) start_height: u16,
}

pub struct Application<'a> {
    is_running: bool, // true
    is_suspended: bool, // false
    window_width: u16,
    window_height: u16,
    ptime: PTime, // system to keep track of time
    game_instance: &'a mut GameInstance,
}

impl<'a> Application<'a> {
    pub(crate) fn create(game_instance: &'a mut GameInstance) -> Self {
        logger::init_logger();

        let width = game_instance.app_config.start_width;
        let height = game_instance.app_config.start_width;




        game_instance.init();
        Self {
            is_running: true,
            is_suspended: false,
            window_width: width,
            window_height: height,
            ptime: PTime::create_system(),
            game_instance
        }
    }


    pub(crate) fn run(&mut self) -> Result<(), String> {
        let mut destroying = false; // true when started the process of destroying vulkan instances


        // on the update event
        {
            self.ptime.tick();

            self.game_instance.update(self.ptime.resource().delta().clone())?;

            self.game_instance.render(self.ptime.resource().delta().clone())?;
        }

        // event_loop.run(move |event, _, control_flow| {
        //     *control_flow = ControlFlow::Poll;
        //
        //     match event {
        //         // Window events ------------
        //         Event::WindowEvent { event, .. } => match event {
        //             WindowEvent::CloseRequested => {
        //                 log::info!("Close requested, closing...");
        //                 *control_flow = ControlFlow::Exit;
        //
        //                 // app.destroy();
        //
        //             }
        //             WindowEvent::KeyboardInput { input, .. } => match input {
        //                 KeyboardInput {
        //                     virtual_keycode,
        //                     state,
        //                     ..
        //                 } => match (virtual_keycode, state) {
        //                     (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
        //                         *control_flow = ControlFlow::Exit
        //                     }
        //                     _ => {}
        //                 },
        //             },
        //             _ => {}
        //         },
        //         // Other events ------------
        //         // render next frame if app is not being destroyed
        //         Event::MainEventsCleared => {
        //             self.ptime.tick();
        //
        //             self.game_instance.update(self.ptime.resource().delta());
        //
        //             self.game_instance.render(self.ptime.resource().delta());
        //         }
        //         Event::RedrawRequested(_window_id) => {
        //
        //             let (new_height, new_width) = (1000, 1000);
        //             self.game_instance.on_window_resize(new_height, new_width);
        //             window.request_redraw();
        //         }
        //         _ => (),
        //     }
        // });

        Ok(())
    }

    fn shutdown(){

    }
}

// let app = AppState {};

// let initialized: bool = ;


// init logging
