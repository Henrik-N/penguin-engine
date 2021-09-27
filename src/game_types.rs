use core;
use crate::core::application::ApplicationConfig;

trait Resource<T> {

}

trait System<T> {
    fn update(resource: &dyn Resource<T>) {

    }
}

pub struct GameInstance {
    pub(crate) app_config: ApplicationConfig,
    // systems: Vec<dyn System<dyn std::any::Any>>,
}

impl GameInstance {
    pub fn create(app_config: ApplicationConfig) -> Self {
        Self {
            app_config,
            // systems: Vec::new(),
        }
    }

    pub fn init(&mut self) {}

    pub fn update(&mut self, delta: f32) -> Result<(), String> {
        // Err(String::from(("Yeet")))
        Ok(())
    }

    pub fn render(&mut self, delta: f32) -> Result<(), String> {
        Ok(())
    }

    pub fn on_window_resize(&mut self, new_width: u16, new_height: u16) {}
}


