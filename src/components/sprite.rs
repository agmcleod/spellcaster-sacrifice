use specs::{Component, VecStorage};

pub struct Sprite {
    pub frame_name: String,
}

impl Sprite {
    pub fn new(frame_name: &str) -> Self {
        Sprite {
            frame_name: frame_name.to_string(),
        }
    }
}

impl Clone for Sprite {
    fn clone(&self) -> Self {
        Sprite {
            frame_name: self.frame_name.clone(),
        }
    }
}

impl Component for Sprite {
    type Storage = VecStorage<Sprite>;
}
