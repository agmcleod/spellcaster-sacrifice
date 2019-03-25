use specs::World;
use std::collections::HashMap;

use crate::components::screen_change::ScreenChange;

pub mod play;

pub trait Screen {
    fn setup(&mut self, world: &mut World);
    fn update(&mut self, world: &mut World);
    fn handle_custom_change(&mut self, action: &String, world: &mut World);
}

pub struct ScreenManager {
    current_state: String,
    states: HashMap<String, Box<Screen>>,
    pub restart_next_frame: bool,
}

impl ScreenManager {
    pub fn new() -> ScreenManager {
        ScreenManager {
            current_state: "".to_string(),
            states: HashMap::new(),
            restart_next_frame: false,
        }
    }

    pub fn add_state(&mut self, name: String, screen: Box<Screen>) {
        self.states.insert(name, screen);
    }

    pub fn cleanup_state(&self, world: &mut World) {
        world.delete_all();
    }

    pub fn process_state_change(&mut self, screen_change: &mut ScreenChange, world: &mut World) {
        if screen_change.action != "" && screen_change.screen != "" {
            if screen_change.action == "restart" {
                self.restart_current_state(world);
            } else if screen_change.action == "start" {
                self.swap_state(screen_change.screen.clone(), world);
            } else {
                self.states
                    .get_mut(&self.current_state)
                    .unwrap()
                    .handle_custom_change(&screen_change.action, world);
            }
        }
    }

    pub fn restart_current_state(&mut self, world: &mut World) {
        self.cleanup_state(world);
        if let Some(current_state) = self.states.get_mut(&self.current_state) {
            current_state.setup(world);
            world.maintain();
        }
    }

    pub fn swap_state(&mut self, name: String, world: &mut World) {
        self.cleanup_state(world);
        self.current_state = name;
        self.states
            .get_mut(&self.current_state)
            .unwrap()
            .setup(world);

        world.maintain();
    }

    pub fn update(&mut self, world: &mut World) {
        self.states
            .get_mut(&self.current_state)
            .unwrap()
            .update(world);
    }
}
