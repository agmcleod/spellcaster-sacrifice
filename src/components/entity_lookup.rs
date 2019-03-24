use std::collections::HashMap;

use specs::Entity;

#[derive(Default)]
pub struct EntityLookup {
    pub entities: HashMap<String, Entity>,
}

impl EntityLookup {
    pub fn new() -> EntityLookup {
        EntityLookup {
            entities: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&Entity> {
        self.entities.get(key)
    }

    pub fn insert(&mut self, key: &str, entity: Entity) {
        self.entities.insert(key.to_string(), entity);
    }
}
