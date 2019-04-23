use specs::{world::Builder, Entity, World};
use tiled::{Group, Map, ObjectGroup, ObjectShape, PropertyValue};

use crate::components::{AnimationSheet, Node, Sprite, Transform};

fn add_group_to_world(
    world: &mut World,
    parent: Option<Entity>,
    group: &Group,
    depth: f32,
) -> Entity {
    let mut group_node = Node::new();
    group_node.parent = parent;
    // width/height here doesnt matter, not using view clipping
    let transform = Transform::visible(group.offset_x, group.offset_y, depth, 1, 1);

    let entity = world
        .create_entity()
        .with(group_node)
        .with(transform)
        .build();

    let mut children = Vec::new();

    for child_layer in &group.children {
        if let Some(object_group) = child_layer.downcast_ref::<ObjectGroup>() {
            for object in &object_group.objects {
                let transform = match object.shape {
                    ObjectShape::Rect { width, height } => {
                        Transform::visible(object.x, object.y, depth, width as u16, height as u16)
                    }
                    _ => Transform::visible(object.x, object.y, depth, 1, 1),
                };

                let mut builder = world.create_entity().with(transform);

                if object.properties.contains_key("frame") {
                    let frame_name = object.properties.get("frame").unwrap();
                    if let PropertyValue::StringValue(frame_name) = frame_name {
                        builder = builder.with(Sprite::new(&frame_name));
                    }
                }

                for (key, value) in &object.properties {
                    if key.starts_with("animation_") {
                        if let PropertyValue::StringValue(frames) = value {
                            let mut animation = AnimationSheet::new(0.1);
                            let frames: Vec<String> = frames
                                .split(",")
                                .map(|frame| format!("{}.png", frame))
                                .collect();
                            animation
                                .add_animation(key.replace("animation_", "").to_string(), frames);

                            builder = builder.with(animation);
                        }
                    }
                }

                children.push(builder.build());
            }
        } else if let Some(group) = child_layer.downcast_ref::<Group>() {
            children.push(add_group_to_world(world, Some(entity), group, depth + 1.0));
        }
    }

    let mut nodes = world.write_storage::<Node>();
    let group_node = nodes.get_mut(entity).unwrap();
    group_node.add_many(children);

    entity
}

pub fn build_from_map(world: &mut World, map: &Map) -> Vec<Entity> {
    map.groups
        .iter()
        .map(|group| add_group_to_world(world, None, group, 1.0))
        .collect()
}
