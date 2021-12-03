use crate::ecs::*;

use legion::Schedule;
use std::collections::HashSet;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Default)]
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn startup(&mut self, resources: &mut Resources) -> Vec<Step> {
        resources.insert(InputEvents::default());
        vec![]
    }

    fn run() -> Vec<Step> {
        Schedule::builder()
            .add_system(update_input_system())
            .build()
            .into_vec()
    }

    fn shutdown() -> Vec<Step> {
        vec![]
    }
}

#[system]
fn update_input(#[resource] input: &mut InputEvents) {
    input.just_released_keys.clear();

    // move just pressed into held down
    input.held_down_keys.extend(input.just_pressed_keys.iter());
    input.just_pressed_keys.clear();

    if let Some(new) = input.new_event {
        match new {
            KeyboardInput {
                virtual_keycode,
                state,
                ..
            } => {
                if let Some(vk) = virtual_keycode {
                    match state {
                        ElementState::Released => {
                            // only add to just released keys if it was actually pressed
                            //  (this ignores duplicate release events)
                            if input.held_down_keys.remove(&vk) {
                                input.just_released_keys.insert(vk);
                            }
                        }
                        ElementState::Pressed => {
                            // if not already pressed, add to just pressed
                            if !input.held_down_keys.contains(&vk) {
                                input.just_pressed_keys.insert(vk);
                                println!("just pressed keys: {:?}", input.just_pressed_keys);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// resource
#[derive(Debug)]
pub(crate) struct InputEvents {
    new_event: Option<winit::event::KeyboardInput>,
    previous_event: Option<winit::event::KeyboardInput>,
    just_pressed_keys: HashSet<VirtualKeyCode>,
    held_down_keys: HashSet<VirtualKeyCode>,
    just_released_keys: HashSet<VirtualKeyCode>,
    just_released_consumed: HashSet<VirtualKeyCode>,
}
impl Default for InputEvents {
    fn default() -> Self {
        Self {
            new_event: None,
            previous_event: None,
            just_pressed_keys: HashSet::with_capacity(10),
            held_down_keys: HashSet::with_capacity(10),
            just_released_keys: HashSet::with_capacity(10),
            just_released_consumed: HashSet::with_capacity(10),
        }
    }
}
impl InputEvents {
    // called from event loop
    pub fn update(&mut self, new_event: winit::event::KeyboardInput) {
        self.new_event = Some(new_event);
    }

    fn check_key_state(&self, in_key: VirtualKeyCode, in_state: ElementState) -> bool {
        match self.previous_event {
            None => false,
            Some(kb_input) => match kb_input {
                KeyboardInput {
                    virtual_keycode,
                    state,
                    ..
                } => match (virtual_keycode, state) {
                    (Some(in_key), in_state) => true,
                    _ => false,
                },
                _ => false,
            },
        }
    }

    pub fn is_key_down(&self, key: VirtualKeyCode) -> bool {
        self.held_down_keys.contains(&key)
    }

    pub fn is_key_up(&self, key: VirtualKeyCode) -> bool {
        !self.held_down_keys.contains(&key)
    }

    pub fn is_key_just_down(&self, key: VirtualKeyCode) -> bool {
        self.just_pressed_keys.contains(&key)
    }

    pub fn is_key_just_up(&self, key: VirtualKeyCode) -> bool {
        self.just_released_keys.contains(&key)
    }
}
