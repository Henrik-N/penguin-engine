use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};

pub use device::Device;
pub use event_loop::init_event_loop;
pub use instance::Instance;
pub use pipeline::Pipeline;
use shaders::Shader;
pub use swapchain::Swapchain;


mod device;
pub mod event_loop;
mod instance;
mod pipeline;
pub mod shaders;
mod swapchain;
pub mod window;


pub struct PenguinEngine {
    instance: self::Instance,
    pub device: self::Device,
    pub swapchain: self::Swapchain
}

impl PenguinEngine {

    pub fn init_engine(window: &winit::window::Window) -> PenguinEngine {
        let pe_instance = self::Instance::init();

        let pe_device = self::Device::init(&pe_instance, &window);
        let pe_swapchain = self::Swapchain::init(&pe_instance, &pe_device);



        //let pipeline = pe_device.create_graphics_pipeline(shaders, &PipelineConfig::default());

        // todo Create command pool

        PenguinEngine {
            instance: pe_instance,
            device: pe_device,
            swapchain: pe_swapchain,
        }
    }

    fn draw_frame(&mut self) {
        // todo
    }

    pub fn run(mut self, event_loop: EventLoop<()>, window: winit::window::Window) {
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
                    self.draw_frame();
                }
                _ => (),
            }
        });
    }
}

impl Drop for PenguinEngine {
    fn drop(&mut self) {
        println!("Dropping PenguinEngine.");

        self.swapchain.drop(&self.device);
        self.device.drop();
        self.instance.drop();
    }
}
