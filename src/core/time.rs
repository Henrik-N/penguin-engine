use crate::ecs::Plugin;
use legion::systems::Step;
use legion::*;
use std::time::{Duration, Instant};

#[derive(Default)]
pub struct TimePlugin;

impl Plugin for TimePlugin {
    fn startup(&mut self, resources: &mut Resources) -> Vec<Step> {
        resources.insert(PTime::default());

        vec![]
    }

    fn run() -> Vec<Step> {
        Schedule::builder()
            .add_system(tick_system())
            .build()
            .into_vec()
    }

    fn shutdown() -> Vec<Step> {
        vec![]
    }
}

#[system]
fn tick(#[resource] time: &mut PTime) {
    time.tick();
}

#[derive(Debug)]
pub struct PTime {
    previous_frame_time: Instant,
    delta_time: Duration,
}
impl Default for PTime {
    fn default() -> Self {
        Self {
            previous_frame_time: Instant::now(),
            delta_time: Duration::new(0, 0),
        }
    }
}
impl PTime {
    pub fn delta(&self) -> f32 {
        self.delta_time.as_secs_f32()
    }

    pub fn _delta_f64(&self) -> f64 {
        self.delta_time.as_secs_f64()
    }

    fn tick(&mut self) {
        let delta_time = self.previous_frame_time.elapsed();
        self.previous_frame_time = Instant::now();
        self.delta_time = delta_time;
    }
}
