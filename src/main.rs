#[macro_use]
extern crate gfx;

use std::collections::HashMap;
use std::time;

use cgmath::Vector3;
use gfx::Device;
use gfx_glyph::GlyphBrushBuilder;
use gfx_window_sdl;
use sdl2::{self, event::Event, keyboard::Keycode};
use specs::World;

mod assets;
mod components;
mod entities;
mod loader;
mod renderer;
mod scene_graph;
mod screen;
mod settings;
mod systems;
mod utils;
mod world;

use assets::spritesheet_map::SpritesheetMap;
use components::{
    tiled::TiledMap, AnimationSheet, Camera, Color, DeltaTime, EntityLookup, Input, Node,
    ScreenChange, Shape, Sprite, Text, Transform,
};
use screen::{play::Play, ScreenManager};
use world::{render_from_node, setup_world};

const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 480;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    video_subsystem
        .gl_attr()
        .set_context_profile(sdl2::video::GLProfile::Core);
    video_subsystem.gl_attr().set_context_version(3, 2);

    let builder = video_subsystem.window("Spellcaster - Sacrifice", SCREEN_WIDTH, SCREEN_HEIGHT);

    let (window, _gl_context, mut device, mut factory, main_color, main_depth) =
        gfx_window_sdl::init::<renderer::ColorFormat, renderer::DepthFormat>(
            &video_subsystem,
            builder,
        )
        .map_err(|err| err.to_string())?;

    let mut world = World::new();
    setup_world(&mut world);

    let mut map_tilesets = HashMap::new();
    map_tilesets.insert(
        "tiles.png".to_string(),
        loader::gfx_load_texture("resources/maps/tiles.png", &mut factory).0,
    );

    let demomap = loader::load_map("resources/maps/demomap.tmx");

    let mut tiled_maps = HashMap::new();
    tiled_maps.insert("demomap".to_string(), demomap);

    let mut glyph_brush =
        GlyphBrushBuilder::using_font_bytes(include_bytes!("../resources/Arial.ttf") as &[u8])
            .build(factory.clone());

    let spritesheet_map = SpritesheetMap::new(&mut factory, &["assets"]);

    let mut screen_manager = ScreenManager::new();
    screen_manager.add_state(Play::get_name(), Box::new(Play::new(tiled_maps)));
    screen_manager.swap_state(Play::get_name(), &mut world);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let target = renderer::WindowTargets {
        color: main_color,
        depth: main_depth,
    };
    let mut renderer = renderer::Renderer::new(&mut factory, target.clone());

    let mut events = sdl_context.event_pump().unwrap();
    let mut running = true;
    let mut frame_start = time::Instant::now();

    while running {
        let duration = time::Instant::now() - frame_start;
        frame_start = time::Instant::now();

        {
            let mut dt = world.write_resource::<DeltaTime>();
            dt.dt = utils::get_seconds(&duration);
        }

        // handle events
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    running = false;
                }
                Event::KeyDown { keycode, .. } => {
                    let mut input = world.write_resource::<Input>();
                    input.set_pressed(&keycode.unwrap(), true);
                }
                Event::KeyUp { keycode, .. } => {
                    let mut input = world.write_resource::<Input>();
                    input.set_pressed(&keycode.unwrap(), false);
                }
                _ => {}
            }
        }

        screen_manager.update(&mut world);
        world.maintain();

        encoder.clear(&target.color, [0.1, 0.2, 0.3, 1.0]);
        encoder.clear_depth(&target.depth, 1.0);

        {
            let sprite_storage = world.read_storage::<Sprite>();
            let mut transform_storage = world.write_storage::<Transform>();
            let animation_sheet_storage = world.read_storage::<AnimationSheet>();
            let color_storage = world.read_storage::<Color>();
            let text_storage = world.read_storage::<Text>();
            let shape_storage = world.read_storage::<Shape>();
            let mut node_storage = world.write_storage::<Node>();
            let tiled_map_storage = world.read_storage::<TiledMap>();

            let root_entity = {
                let lookup = world.read_resource::<EntityLookup>();
                lookup.entities.get("root").unwrap().clone()
            };

            let mut offset_position = Vector3::<f32>::new(0.0, 0.0, 0.0);

            render_from_node(
                &mut renderer,
                &mut encoder,
                root_entity,
                &world,
                &mut factory,
                &spritesheet_map,
                &map_tilesets,
                &mut glyph_brush,
                &sprite_storage,
                &mut transform_storage,
                &animation_sheet_storage,
                &color_storage,
                &text_storage,
                &shape_storage,
                &tiled_map_storage,
                &mut node_storage,
                &mut offset_position,
                &(1.0, 1.0),
            );
        }

        // <- draw actual stuff here
        renderer.flush(
            &mut encoder,
            &mut factory,
            &spritesheet_map,
            &world.read_resource::<Camera>(),
            "",
            true,
        );
        encoder.flush(&mut device);
        window.gl_swap_window();
        device.cleanup();

        let mut state_change = {
            let mut state_change_storage = world.write_resource::<ScreenChange>();
            let copy = state_change_storage.clone();
            state_change_storage.reset();
            copy
        };

        screen_manager.process_state_change(&mut state_change, &mut world);
    }

    Ok(())
}
