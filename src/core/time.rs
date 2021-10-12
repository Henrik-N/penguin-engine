use std::time::{Instant, Duration};


// Resource
pub struct PTimeResource {
    delta_time: Duration,
}

impl PTimeResource {
    pub fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn delta_f64(&self) -> f64 {
        self.delta_time.as_secs_f64()
    }
}

// System
pub struct PTime {
    previous_frame_time: Instant,
    resource: PTimeResource,
}

impl PTime {
    pub fn create_system() -> Self {
        Self {
            previous_frame_time: Instant::now(),
            resource: PTimeResource {
                delta_time: Duration::new(0, 0)
            },
        }
    }

    pub fn resource(&self) -> &PTimeResource {
        &self.resource
    }

    pub fn tick(&mut self) {
        let delta_time = self.previous_frame_time.elapsed();
        self.previous_frame_time = Instant::now();
        self.resource.delta_time = delta_time;
    }
}
