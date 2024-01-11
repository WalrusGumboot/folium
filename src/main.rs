#![allow(dead_code)]

mod ast;
mod error;
mod interpreter;
mod layout;
mod render;
mod style;

use std::{fs, path::PathBuf};

use sdl2::{event::Event, image::SaveSurface, keyboard::Keycode};

use clap::{Parser, Subcommand};

pub const SLIDE_WIDTH: u32 = 1920;
pub const SLIDE_HEIGHT: u32 = 1080;

#[derive(Parser)]
#[command(author = "Simeon Duwel", about = "Presentation renderer and viewer")]
struct FoliumArgs {
    /// The source .flm file containing your presentation
    input: PathBuf,
    #[command(subcommand)]
    command: FoliumSubcommand,
}

#[derive(Subcommand)]
enum FoliumSubcommand {
    /// Render out a set of slides as images to a folder
    Render { output: PathBuf },
    /// Open a presentation window
    Present,
    /// Inspect a .flm file and print some info. Can also be used as a check for syntax errors
    Inspect,
}

fn main() {
    let args = FoliumArgs::parse();

    let state = ast::GlobalState::new();
    interpreter::load_from_file(&state, args.input).unwrap();

    let number_of_slides = state.number_of_slides();

    match args.command {
        FoliumSubcommand::Render { output } => {
            assert!(!output.is_file(), "{} is a file", output.display());

            if !output.exists() {
                fs::create_dir(&output).unwrap();
            }

            for i in 0..number_of_slides {
                let dimensions = render::generate_slide_data(&state, i, false).dimensions;
                let surface = sdl2::surface::Surface::new(
                    dimensions.0,
                    dimensions.1,
                    sdl2::pixels::PixelFormatEnum::RGB24,
                )
                .unwrap();
                let mut canvas = surface.into_canvas().unwrap();
                let texture_creator = canvas.texture_creator();

                let texture_map = render::generate_textures(&state, &texture_creator);

                render::render(&state, &mut canvas, i, false, &texture_map);
                canvas
                    .into_surface()
                    .save(output.join(format!("{}.png", i + 1)))
                    .unwrap();
            }
        }
        FoliumSubcommand::Present => {
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

            let mut texture_creator = canvas.texture_creator();
            let texture_map = render::generate_textures(&state, &texture_creator);

            let mut slide_idx: usize = 0;

            'run: loop {
                render::render(&state, &mut canvas, slide_idx, true, &texture_map);
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

                std::thread::sleep(std::time::Duration::from_secs_f64(60f64.recip()))
            }
        }
        FoliumSubcommand::Inspect => {
            println!("{state}");
        }
    }
}
