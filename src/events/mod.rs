// -------------------------- EVENT LOOP ----------------------------
use winit::event_loop::EventLoop;

pub fn init_event_loop() -> EventLoop<()> {
    EventLoop::new()
}
