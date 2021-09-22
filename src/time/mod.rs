use std::time::{Instant, Duration};

pub struct Time {
    previous_frame_time: Instant,
    delta_time: Duration,
}

impl Time {
    pub fn init() -> Self {
        Self {
            previous_frame_time: Instant::now(),
            delta_time: Duration::new(0, 0),
        }
    }

    pub fn tick(&mut self) {
        self.delta_time = self.previous_frame_time.elapsed();
        self.previous_frame_time = Instant::now();
    }

    pub fn delta_time(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn delta_time_f64(&self) -> f64 {
        self.delta_time.as_secs_f64()
    }
}
