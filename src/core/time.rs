use crate::ecs::Plugin;
use std::time::{Duration, Instant};
use legion::systems::Step;
use legion::*;

pub struct TimePlugin;
impl Plugin for TimePlugin {
    fn init_resources(resources: &mut Resources) {
        resources.insert(PTime::default());
    }

    fn startup_steps() -> Vec<Step> {
        vec![]
    }

    fn run_steps() -> Vec<Step> {
        Schedule::builder()
            .add_system(tick_system())
            .build()
            .into_vec()
    }

    fn shutdown_steps() -> Vec<Step> {
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

