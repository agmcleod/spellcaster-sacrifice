use specs::{Component, HashMapStorage};

#[derive(Default)]
pub struct ScreenChange {
    pub screen: String,
    pub action: String,
}

impl Clone for ScreenChange {
    fn clone(&self) -> ScreenChange {
        ScreenChange {
            screen: self.screen.clone(),
            action: self.action.clone(),
        }
    }
}

impl ScreenChange {
    pub fn new() -> ScreenChange {
        ScreenChange {
            screen: "".to_string(),
            action: "".to_string(),
        }
    }

    pub fn reset(&mut self) {
        self.screen = "".to_string();
        self.action = "".to_string();
    }

    pub fn set(&mut self, screen: String, action: String) {
        self.screen = screen;
        self.action = action;
    }
}

impl Component for ScreenChange {
    type Storage = HashMapStorage<ScreenChange>;
}
