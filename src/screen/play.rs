use specs::{Dispatcher, DispatcherBuilder, World};

use super::Screen;

pub struct Play {}

impl Play {
    pub fn get_name() -> String {
        "play".to_string()
    }
}

impl Screen for Play {
    fn setup(&mut self, world: &mut World) {}

    fn update(&mut self, world: &mut World) {}

    fn handle_custom_change(&mut self, action: &String, world: &mut World) {}
}
