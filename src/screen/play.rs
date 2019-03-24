use std::collections::HashMap;

use specs::{world::Builder, Dispatcher, DispatcherBuilder, World};
use tiled::Map;

use crate::components::{entity_lookup::EntityLookup, map::tiled::TiledMap, node::Node};

use super::Screen;

pub struct Play<'a> {
    dispatcher: Dispatcher<'a, 'a>,
    pub tiled_maps: HashMap<String, Map>,
}

impl<'a> Play<'a> {
    pub fn new(tiled_maps: HashMap<String, Map>) -> Self {
        Play {
            dispatcher: DispatcherBuilder::new().build(),
            tiled_maps,
        }
    }

    pub fn get_name() -> String {
        "play".to_string()
    }
}

impl<'a> Screen for Play<'a> {
    fn setup(&mut self, world: &mut World) {
        let root = world.create_entity().with(Node::new()).build();

        world
            .create_entity()
            .with(TiledMap::new(
                self.tiled_maps.get(&"demomap".to_string()).unwrap(),
            ))
            .with(Node::with_parent(root))
            .build();

        let mut lookup = world.write_resource::<EntityLookup>();
        lookup.insert("root", root);
    }

    fn update(&mut self, world: &mut World) {}

    fn handle_custom_change(&mut self, action: &String, world: &mut World) {}
}
