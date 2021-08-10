mod pe;

fn main() {
    // let shaders = pe::shaders::Shaders::init();

    let event_loop = pe::init_event_loop();
    let window = pe::window::init_window(&event_loop);

    let engine = pe::PenguinEngine::init_engine(&window);
    let dev = &engine.device;

    let pipeline = dev.build_graphics_pipeline().build();
    pipeline.drop(&dev);

    engine.run(event_loop, window);
}
