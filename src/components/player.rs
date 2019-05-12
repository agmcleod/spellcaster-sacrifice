use specs::{Component, NullStorage};

#[derive(Default)]
pub struct Player;

impl Player {
    pub fn new() -> Self {
        Player {}
    }
}

impl Component for Player {
    type Storage = NullStorage<Self>;
}
