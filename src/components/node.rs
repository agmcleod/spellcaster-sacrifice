use std::cmp;

use cgmath::Vector3;
use specs::{Component, Entity, ReadStorage, VecStorage, World, WriteStorage};

use crate::components::transform::Transform;

#[derive(Default)]
pub struct Node {
    pub entities: Vec<Entity>,
    pub parent: Option<Entity>,
    pub children_dirty: bool,
}

impl Node {
    pub fn new() -> Self {
        Node {
            entities: Vec::new(),
            parent: None,
            children_dirty: false,
        }
    }

    pub fn with_parent(entity: Entity) -> Self {
        Node {
            entities: Vec::new(),
            parent: Some(entity),
            children_dirty: false,
        }
    }

    pub fn add(&mut self, entity: Entity) {
        self.entities.push(entity);
        self.children_dirty = true;
    }

    pub fn add_many(&mut self, entities: Vec<Entity>) {
        self.entities.extend(entities);
        self.children_dirty = true;
    }

    pub fn sort_children<'a>(
        &mut self,
        world: &World,
        transform_storage: &WriteStorage<'a, Transform>,
    ) {
        let mut removed = false;
        // cleans up nodes for us, so we dont have to do it manually
        self.entities.retain(|e| {
            if world.is_alive(*e) {
                return true;
            }

            removed = true;
            false
        });

        if removed {
            self.children_dirty = true;
        }

        if self.children_dirty {
            self.entities.sort_by(|entity_a, entity_b| {
                let transform_a = if let Some(t) = transform_storage.get(*entity_a) {
                    t
                } else {
                    return cmp::Ordering::Greater;
                };

                let transform_b = if let Some(t) = transform_storage.get(*entity_b) {
                    t
                } else {
                    return cmp::Ordering::Greater;
                };

                (transform_a.get_pos().z as i32).cmp(&(transform_b.get_pos().z as i32))
            });
            self.children_dirty = false;
        }
    }
}

impl Component for Node {
    type Storage = VecStorage<Self>;
}
