use penguin_app::{config::AppConfig, App};
use penguin_config::PenguinConfig;

fn main() {
    App::builder(AppConfig::read_config())
        .add_plugin(penguin_app::time_plugin::TimePlugin)
        .add_plugin(penguin_renderer::renderer::RendererPlugin)
        .run()
        .expect("app run loop failed");
}
