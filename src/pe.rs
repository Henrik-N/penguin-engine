

pub use device::Device;
// pub use r#mod::init_event_loop;
pub use instance::Instance;
pub use pipeline::Pipeline;
use shaders::Shader;
pub use swapchain::Swapchain;


mod device;
// pub mod event_loop;
mod instance;
mod pipeline;
pub mod shaders;
mod swapchain;
pub mod window;


pub struct PenguinEngine {
    instance: self::Instance,
    pub device: self::Device,
    pub swapchain: self::Swapchain,
    seconds_passed: f32,
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
            seconds_passed: 0.
        }
    }

    pub fn draw_frame(&mut self, delta_time: f32) {
        self.seconds_passed += delta_time;
        println!("Seconds passed: {}", self.seconds_passed);
    }

    // pub fn run(mut self, event_loop: EventLoop<()>, window: winit::window::Window) {
    //
    // }
}

impl Drop for PenguinEngine {
    fn drop(&mut self) {
        println!("Dropping PenguinEngine.");

        self.swapchain.drop(&self.device);
        self.device.drop();
        self.instance.drop();
    }
}
