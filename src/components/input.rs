use sdl2::keyboard::Keycode;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct Input {
    pub actions: HashMap<String, HashSet<Keycode>>,
    pub hidpi_factor: f32,
    pub pressed_keys: HashMap<Keycode, bool>,
    pub mouse_pos: (f32, f32),
    pub mouse_pressed: bool,
}

impl Input {
    pub fn new(hidpi_factor: f32, actions: HashMap<String, HashSet<Keycode>>) -> Input {
        let mut key_map: HashMap<Keycode, bool> = HashMap::new();
        for (_, codes) in &actions {
            for code in codes {
                key_map.insert(*code, false);
            }
        }

        Input {
            actions,
            hidpi_factor: hidpi_factor,
            pressed_keys: key_map,
            mouse_pos: (0.0, 0.0),
            mouse_pressed: false,
        }
    }

    pub fn is_pressed(&self, action: &str) -> bool {
        if let Some(keys) = self.actions.get(action) {
            keys.iter().fold(false, |result, key| {
                if let Some(pressed) = self.pressed_keys.get(key) {
                    *pressed || result
                } else {
                    panic!("Could not find key {:?} in pressed state", key);
                }
            })
        } else {
            panic!("action {} not recognized", action);
        }
    }

    pub fn set_pressed(&mut self, code: &Keycode, pressed: bool) {
        if let Some(key) = self.pressed_keys.get_mut(code) {
            *key = pressed;
        }
    }
}
