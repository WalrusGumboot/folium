#![allow(dead_code)]

mod ast;
mod error;
mod interpreter;
mod layout;
mod style;
mod render;

use sdl2::{event::Event, keyboard::Keycode};

fn main() {
    let state = ast::GlobalState::new();
    interpreter::load_from_file(&state, "test.flm").unwrap();

    let sdl_context = sdl2::init().expect("Could not create SDL2 context");
    let vid_context = sdl_context.video().expect("Could not create video context");
    let window = vid_context
        .window("folium", 1920, 1080)
        .fullscreen_desktop()
        .input_grabbed()
        .position_centered()
        .borderless()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut slide_idx: usize = 0;
    let number_of_slides = state.number_of_slides();

    'run: loop {
        render::render(&state, &mut canvas, slide_idx);

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'run,
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    slide_idx = (number_of_slides - 1).min(slide_idx + 1);
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    slide_idx = slide_idx.saturating_sub(1);
                }
                _ => {}
            }
        }
    }
}
