#![allow(unused)]
use anyhow::*;

use penguin_config::PenguinConfig;
use penguin_app::{App, AppBuilder, config::AppConfig};



fn main() -> Result<()> {
    App::builder(AppConfig::read_config()
    )
        .add_plugin(penguin_app::time_plugin::TimePlugin)
        .add_plugin(penguin_renderer::renderer::RendererPlugin)
        .run()?;


    Ok(())
}
