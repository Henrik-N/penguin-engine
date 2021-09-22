use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

mod pe;
mod time;
mod events;

fn main() {

    //let shaders = pe::shaders::Shaders::init();

    let mut time = time::Time::init();


    let event_loop = events::init_event_loop();
    let window = pe::window::init_window(&event_loop);

    let mut engine = pe::PenguinEngine::init_engine(&window);
    let device = &engine.device;

    let pipeline = device.build_graphics_pipeline().build();
    pipeline.drop(&device);


    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    println!("Close requested, closing...");
                    *control_flow = ControlFlow::Exit
                }
                WindowEvent::KeyboardInput { input, .. } => match input {
                    KeyboardInput {
                        virtual_keycode,
                        state,
                        ..
                    } => match (virtual_keycode, state) {
                        (Some(VirtualKeyCode::Escape), ElementState::Pressed) => {
                            *control_flow = ControlFlow::Exit
                        }
                        _ => {}
                    },
                },
                _ => {}
            },
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::RedrawRequested(_window_id) => {
                time.tick();

                engine.draw_frame(time.delta_time());
            }
            _ => (),
        }
    });

    // engine.run(event_loop, window);
}
