use cgmath::Vector3;
use gfx_glyph::GlyphBrush;
use sdl2::keyboard::Keycode;
use serde_json::{self, Value};
use specs::{Entity, ReadStorage, World, WriteStorage};
use std::collections::{HashMap, HashSet};

use crate::loader::read_text_from_file;

use crate::{
    assets::spritesheet_map::SpritesheetMap,
    components::{
        tiled::TiledMap, AnimationSheet, Camera, Color, DeltaTime, EntityLookup, Input, Node,
        Player, ScreenChange, Shape, Sprite, Text, Transform,
    },
    loader::Texture,
    renderer::{get_ortho, Renderer},
};

pub fn setup_world(world: &mut World) {
    world.add_resource(DeltaTime::default());
    world.add_resource(Camera(get_ortho()));
    world.add_resource(EntityLookup::new());

    let keybindings: Value = serde_json::from_str(
        &read_text_from_file("resources/keybindings.json")
            .map_err(|_| panic!("Could not find keybindings"))
            .unwrap(),
    )
    .unwrap();
    let keybindings = keybindings
        .as_object()
        .unwrap_or_else(|| panic!("Could not parse keybindings file into object"));

    let mut actions = HashMap::new();

    for (action, keys) in keybindings {
        let mut key_codes = HashSet::new();
        let keys = keys.as_array().unwrap();
        for key in keys {
            if let Some(key_code) = Keycode::from_name(
                key.as_str()
                    .unwrap_or_else(|| panic!("Key {} in keybindings file not string type", key)),
            ) {
                key_codes.insert(key_code);
            } else {
                panic!("Could not parse Key: {}", key);
            }
        }

        actions.insert(action.clone(), key_codes);
    }

    world.add_resource(Input::new(1.0, actions));

    world.add_resource(ScreenChange::new());

    world.register::<AnimationSheet>();
    world.register::<Color>();
    world.register::<Node>();
    world.register::<Player>();
    world.register::<Shape>();
    world.register::<Sprite>();
    world.register::<Text>();
    world.register::<TiledMap>();
    world.register::<Transform>();
}

fn render_entity<R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(
    renderer: &mut Renderer<R>,
    encoder: &mut gfx::Encoder<R, C>,
    world: &World,
    factory: &mut F,
    spritesheet: &SpritesheetMap<R>,
    map_tilesets: &HashMap<String, Texture<R>>,
    glyph_brush: &mut GlyphBrush<R, F>,
    entity: &Entity,
    sprite_storage: &ReadStorage<Sprite>,
    transform_storage: &mut WriteStorage<Transform>,
    animation_storage: &ReadStorage<AnimationSheet>,
    color_storage: &ReadStorage<Color>,
    text_storage: &ReadStorage<Text>,
    shape_storage: &ReadStorage<Shape>,
    tiled_map_storage: &ReadStorage<TiledMap>,
    offset_position: &mut Vector3<f32>,
    scale_from_base_res: &(f32, f32),
) {
    if let Some(transform) = transform_storage.get(*entity) {
        if let Some(sprite) = sprite_storage.get(*entity) {
            renderer.render(
                encoder,
                world,
                factory,
                &transform,
                Some(&sprite.frame_name),
                spritesheet,
                color_storage.get(*entity),
                offset_position,
            );
        }

        if let Some(animation) = animation_storage.get(*entity) {
            renderer.render(
                encoder,
                world,
                factory,
                &transform,
                Some(animation.get_current_frame()),
                spritesheet,
                color_storage.get(*entity),
                offset_position,
            );
        }

        if let (Some(color), Some(text)) = (color_storage.get(*entity), text_storage.get(*entity)) {
            if text.text != "" && text.visible {
                renderer.render_text(
                    encoder,
                    &text,
                    transform,
                    color,
                    glyph_brush,
                    world.read_resource::<Input>().hidpi_factor,
                    scale_from_base_res,
                    offset_position,
                );
            }
        }

        if let Some(shape) = shape_storage.get(*entity) {
            renderer.render_shape(encoder, world, factory, &shape);
        }

        if let Some(tile_map) = tiled_map_storage.get(*entity) {
            if let Some(texture) = map_tilesets.get(&tile_map.tileset) {
                renderer.draw_batch(
                    &tile_map.data,
                    encoder,
                    world,
                    factory,
                    spritesheet,
                    &tile_map.tileset,
                    texture,
                );
            } else {
                panic!("Could not find texture by name {}", tile_map.tileset);
            }
        }
    }
}

pub fn render_from_node<R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(
    renderer: &mut Renderer<R>,
    encoder: &mut gfx::Encoder<R, C>,
    entity: Entity,
    world: &World,
    factory: &mut F,
    spritesheet: &SpritesheetMap<R>,
    map_tilesets: &HashMap<String, Texture<R>>,
    glyph_brush: &mut GlyphBrush<R, F>,
    sprite_storage: &ReadStorage<Sprite>,
    transform_storage: &mut WriteStorage<Transform>,
    animation_sheet_storage: &ReadStorage<AnimationSheet>,
    color_storage: &ReadStorage<Color>,
    text_storage: &ReadStorage<Text>,
    shape_storage: &ReadStorage<Shape>,
    tiled_map_storage: &ReadStorage<TiledMap>,
    node_storage: &mut WriteStorage<Node>,
    offset_position: &mut Vector3<f32>,
    scale_from_base_res: &(f32, f32),
) {
    if let Some(transform) = transform_storage.get(entity) {
        if !transform.visible {
            return;
        }

        let pos = transform.get_pos();
        offset_position.x += pos.x;
        offset_position.y += pos.y;
        offset_position.z += pos.z;
    }

    render_entity(
        renderer,
        encoder,
        world,
        factory,
        spritesheet,
        map_tilesets,
        glyph_brush,
        &entity,
        sprite_storage,
        transform_storage,
        animation_sheet_storage,
        color_storage,
        text_storage,
        shape_storage,
        tiled_map_storage,
        offset_position,
        scale_from_base_res,
    );

    let mut entities = Vec::new();
    if let Some(node) = node_storage.get_mut(entity) {
        node.sort_children(world, transform_storage);
        entities.append(&mut node.entities.iter().cloned().collect());
    }

    for entity in &entities {
        render_from_node(
            renderer,
            encoder,
            *entity,
            world,
            factory,
            spritesheet,
            map_tilesets,
            glyph_brush,
            sprite_storage,
            transform_storage,
            animation_sheet_storage,
            color_storage,
            text_storage,
            shape_storage,
            tiled_map_storage,
            node_storage,
            offset_position,
            scale_from_base_res,
        );
    }

    if let Some(transform) = transform_storage.get(entity) {
        let pos = transform.get_pos();
        offset_position.x -= pos.x;
        offset_position.y -= pos.y;
        offset_position.z -= pos.z;
    }
}
