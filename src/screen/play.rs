use std::collections::HashMap;

use specs::{world::Builder, Dispatcher, DispatcherBuilder, World};
use tiled::Map;

use crate::{
    components::{tiled::TiledMap, EntityLookup, Node, Transform},
    entities,
    systems::{AnimationSystem, PlayerSystem},
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
            dispatcher: DispatcherBuilder::new()
                .with(AnimationSystem::new(), "animation", &[])
                .with(PlayerSystem::new(), "player", &[])
                .build(),
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

        let demomap = self.tiled_maps.get(&"demomap".to_string()).unwrap();

        let entity = world
            .create_entity()
            .with(TiledMap::new(demomap))
            .with(Node::with_parent(root))
            .with(Transform::visible(
                0.0,
                0.0,
                0.0,
                SCREEN_WIDTH as u16,
                SCREEN_HEIGHT as u16,
            ))
            .build();
        children.push(entity);

        let entities_from_map = entities::build_from_map(world, &demomap);

        let mut node_storage = world.write_storage::<Node>();
        let node = node_storage.get_mut(root).unwrap();
        node.add_many(children);
        node.add_many(entities_from_map);

        let mut lookup = world.write_resource::<EntityLookup>();
        lookup.insert("root", root);
    }

    fn update(&mut self, world: &mut World) {
        self.dispatcher.dispatch(&mut world.res);
    }

    fn handle_custom_change(&mut self, action: &String, world: &mut World) {}
}
