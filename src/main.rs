use std::time;

use gfx::{
    self,
    format::{DepthStencil, Rgba8},
    Device,
};
use gfx_window_sdl;
use sdl2::{self, event::Event, keyboard::Keycode};
use specs::World;

mod assets;
mod components;
mod loader;
mod screen;
mod settings;
mod utils;
mod world;

use components::delta_time::DeltaTime;
use world::setup_world;

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    video_subsystem
        .gl_attr()
        .set_context_profile(sdl2::video::GLProfile::Core);
    video_subsystem.gl_attr().set_context_version(3, 2);

    let builder = video_subsystem.window("Spellcaster - Sacrifice", 1280, 720);

    let (window, _gl_context, mut device, mut factory, main_color, _main_depth) =
        gfx_window_sdl::init::<Rgba8, DepthStencil>(&video_subsystem, builder)
            .map_err(|err| err.to_string())?;

    let mut world = World::new();

    setup_world(&mut world);

    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

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

        // draw a frame
        encoder.clear(&main_color, [0.1, 0.2, 0.3, 1.0]);
        // <- draw actual stuff here
        encoder.flush(&mut device);
        window.gl_swap_window();
        device.cleanup();
    }

    Ok(())
}
