pub use legion::systems::Resources;
pub use legion::systems::Step;
pub use legion::*;

/// Group of behaviour
pub trait Plugin {
    /// resources.insert(ResourceType::default());
    /// Schedule::builder().add_system(some_system_system()).build().into_vec();
    fn startup(&mut self, resources: &mut Resources) -> Vec<Step>;

    /// Schedule::builder().add_system(some_system_system()).build().into_vec()
    fn run() -> Vec<Step>;

    /// Schedule::builder().add_system(some_system_system()).build().into_vec()
    fn shutdown() -> Vec<Step>;
}

pub struct AppBuilder {
    world: World,
    resources: Resources,
    startup_steps: Vec<Step>,
    run_steps: Vec<Step>,
    shutdown_steps: Vec<Step>,
}

impl AppBuilder {
    pub fn builder() -> Self {
        Self {
            world: World::default(),
            resources: Resources::default(),
            startup_steps: Vec::new(),
            run_steps: Vec::new(),
            shutdown_steps: Vec::new(),
        }
    }

    pub fn add_startup_steps(mut self, steps: Vec<Step>) -> Self {
        self.startup_steps.extend(steps.into_iter());
        self
    }

    pub fn add_run_steps(mut self, steps: Vec<Step>) -> Self {
        self.run_steps.extend(steps.into_iter());
        self
    }

    pub fn add_shutdown_steps(mut self, steps: Vec<Step>) -> Self {
        self.shutdown_steps.extend(steps.into_iter());
        self
    }

    pub fn insert_resource<T>(mut self, resource: T) -> Self
    where
        T: 'static,
    {
        self.resources.insert(resource);
        self
    }

    pub fn add_plugin<T: Plugin>(mut self, mut plugin: T) -> Self {
        let startup_steps = plugin.startup(&mut self.resources);
        self = self.add_startup_steps(startup_steps);
        self = self.add_run_steps(T::run());
        self = self.add_shutdown_steps(T::shutdown());
        self
    }

    pub fn build(self) -> crate::App {
        let startup_schedule = Schedule::from(self.startup_steps);
        let run_schedule = Schedule::from(self.run_steps);
        let shutdown_schedule = Schedule::from(self.shutdown_steps);

        crate::App {
            world: self.world,
            resources: self.resources,

            startup_schedule,
            run_schedule,
            shutdown_schedule,
        }
    }
}
