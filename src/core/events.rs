// -------------------------- EVENT LOOP ----------------------------
use winit::event_loop::EventLoop;

struct EventSender {

}

impl EventSender {

}

trait EventP {

}


// struct EventSystem {
//     event_loop: winit::event_loop::EventLoop<()>,
// }



pub fn init_event_loop() -> EventLoop<()> {
    EventLoop::new()
}
