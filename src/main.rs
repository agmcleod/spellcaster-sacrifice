#[macro_use]
extern crate gfx;

use std::time;

use gfx::{
    format::{DepthStencil, Rgba8},
    Device,
};
use gfx_window_sdl;
use sdl2::{self, event::Event, keyboard::Keycode};
use specs::World;

mod assets;
mod components;
mod loader;
mod renderer;
mod scene_graph;
mod screen;
mod settings;
mod utils;
mod world;

use components::{delta_time::DeltaTime, screen_change::ScreenChange};
use screen::{play::Play, ScreenManager};
use world::setup_world;

const SCREEN_WIDTH: u32 = 1280;
const SCREEN_HEIGHT: u32 = 720;

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

    let mut screen_manager = ScreenManager::new();
    screen_manager.add_state(Play::get_name(), Box::new(Play {}));
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
                _ => {}
            }
        }

        screen_manager.update(&mut world);

        // draw a frame
        encoder.clear(&target.color, [0.1, 0.2, 0.3, 1.0]);
        // <- draw actual stuff here
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
