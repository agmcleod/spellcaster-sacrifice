use cgmath::Vector3;
use specs::{Entity, WriteStorage};

use crate::components::{Node, Transform};

pub fn get_absolute_pos(
    entity: Entity,
    node_storage: &WriteStorage<Node>,
    transform_storage: &WriteStorage<Transform>,
) -> Vector3<f32> {
    let mut pos = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };

    let mut next_entity = entity;
    loop {
        if let Some(transform) = transform_storage.get(next_entity) {
            let local_pos = transform.get_pos();
            pos.x += local_pos.x;
            pos.y += local_pos.y;
            pos.z += local_pos.z;
        }
        if let Some(node) = node_storage.get(next_entity) {
            if let Some(parent) = node.parent {
                next_entity = parent;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    pos
}
