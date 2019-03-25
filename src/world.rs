use gfx_glyph::GlyphBrush;
use sdl2::keyboard::Keycode;
use specs::{Entity, ReadStorage, World, WriteStorage};
use std::collections::HashMap;

use crate::{
    assets::spritesheet_map::SpritesheetMap,
    components::{
        animation_sheet::AnimationSheet, camera::Camera, color::Color, delta_time::DeltaTime,
        entity_lookup::EntityLookup, input::Input, map::tiled::TiledMap, node::Node,
        screen_change::ScreenChange, shape::Shape, sprite::Sprite, text::Text,
        transform::Transform,
    },
    loader::Texture,
    renderer::{get_ortho, Renderer},
};

pub fn setup_world(world: &mut World) {
    world.add_resource(DeltaTime::default());
    world.add_resource(Camera(get_ortho()));
    world.add_resource(EntityLookup::new());
    world.add_resource(Input::new(1.0, vec![Keycode::Escape]));
    world.add_resource(ScreenChange::new());

    world.register::<AnimationSheet>();
    world.register::<Color>();
    world.register::<Node>();
    world.register::<Shape>();
    world.register::<Sprite>();
    world.register::<Text>();
    world.register::<TiledMap>();
    world.register::<Transform>();
}

pub fn render_entity<R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(
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
    scale_from_base_res: &(f32, f32),
) {
    if let Some(transform) = transform_storage.get_mut(*entity) {
        if transform.visible {
            if let Some(sprite) = sprite_storage.get(*entity) {
                renderer.render(
                    encoder,
                    world,
                    factory,
                    &transform,
                    Some(&sprite.frame_name),
                    spritesheet,
                    color_storage.get(*entity),
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
                );
            }

            if let (Some(color), Some(text)) =
                (color_storage.get(*entity), text_storage.get(*entity))
            {
                if text.text != "" && text.visible {
                    renderer.render_text(
                        encoder,
                        &text,
                        transform,
                        color,
                        glyph_brush,
                        world.read_resource::<Input>().hidpi_factor,
                        scale_from_base_res,
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
}

pub fn prepare_node<R: gfx::Resources, C: gfx::CommandBuffer<R>, F: gfx::Factory<R>>(
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
    scale_from_base_res: &(f32, f32),
) {
    if let Some(transform) = transform_storage.get(entity) {
        if !transform.visible {
            return;
        }
        renderer.transform(&transform, false);
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
        scale_from_base_res,
    );

    let mut entities = Vec::new();
    if let Some(node) = node_storage.get_mut(entity) {
        node.sort_children(world, transform_storage);
        entities.append(&mut node.entities.iter().cloned().collect());
    }

    for entity in &entities {
        prepare_node(
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
            scale_from_base_res,
        );
    }

    if let Some(transform) = transform_storage.get(entity) {
        renderer.transform(&transform, true);
    }
}
