use std::collections::HashMap;

use specs::{world::Builder, Dispatcher, DispatcherBuilder, World};
use tiled::Map;

use crate::{
    components::{
        entity_lookup::EntityLookup, map::tiled::TiledMap, node::Node, transform::Transform,
    },
    SCREEN_HEIGHT, SCREEN_WIDTH,
};

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
        let root = world
            .create_entity()
            .with(Transform::visible(
                0.0,
                0.0,
                0.0,
                SCREEN_WIDTH as u16,
                SCREEN_HEIGHT as u16,
            ))
            .with(Node::new())
            .build();

        let mut children = Vec::new();

        let entity = world
            .create_entity()
            .with(TiledMap::new(
                self.tiled_maps.get(&"demomap".to_string()).unwrap(),
            ))
            .with(Node::with_parent(root))
            .build();

        children.push(entity);

        let mut node_storage = world.write_storage::<Node>();
        let node = node_storage.get_mut(root).unwrap();
        node.add_many(children);

        let mut lookup = world.write_resource::<EntityLookup>();
        lookup.insert("root", root);
    }

    fn update(&mut self, world: &mut World) {}

    fn handle_custom_change(&mut self, action: &String, world: &mut World) {}
}
