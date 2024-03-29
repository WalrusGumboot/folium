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
    #[arg(long, short, default_value_t = false, global = true)]
    /// Whether or not to draw red 1px rectangles around all elements; useful for debugging layout issues
    rects: bool,
    #[command(subcommand)]
    command: FoliumSubcommand,
}

#[derive(Subcommand)]
enum FoliumSubcommand {
    /// Render out a set of slides as images to a folder
    Render {
        /// The source .flm file containing your presentation
        input: PathBuf,
        /// The directory path to write the files to
        output: PathBuf,
    },
    /// Open a presentation window
    Present {
        /// The source .flm file containing your presentation
        input: PathBuf,
    },
    /// Inspect a .flm file and print some info. Can also be used as a check for syntax errors
    Inspect {
        /// The source .flm file containing your presentation
        input: PathBuf,
    },
    /// Lists all possible font values available for styling.
    #[command(subcommand_negates_reqs = true)]
    ListFonts,
}

fn main() {
    let args = FoliumArgs::parse();

    match args.command {
        FoliumSubcommand::Render { input, output } => {
            let state = ast::GlobalState::new();
            interpreter::load_from_file(&state, input).unwrap();

            let number_of_slides = state.number_of_slides();

            assert!(!output.is_file(), "{} is a file", output.display());

            if !output.exists() {
                fs::create_dir(&output).unwrap();
            }

            for i in 0..number_of_slides {
                let dimensions = render::generate_slide_data(&state, i, false).dimensions;
                let surface = sdl2::surface::Surface::new(
                    dimensions.0,
                    dimensions.1,
                    sdl2::pixels::PixelFormatEnum::RGBA32,
                )
                .unwrap();
                let mut canvas = surface.into_canvas().unwrap();
                canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

                let texture_creator = canvas.texture_creator();
                let rendering_data = render::initialise_rendering_data(&state, &texture_creator);

                render::render(&state, &mut canvas, i, false, &rendering_data, args.rects);
                canvas
                    .into_surface()
                    .save(output.join(format!("{}.png", i + 1)))
                    .unwrap();
            }
        }
        FoliumSubcommand::Present { input } => {
            let state = ast::GlobalState::new();
            interpreter::load_from_file(&state, input).unwrap();

            let number_of_slides = state.number_of_slides();

            let sdl_context = sdl2::init().expect("Could not create SDL2 context");
            let vid_context = sdl_context.video().expect("Could not create video context");
            let window = vid_context
                .window("folium", SLIDE_WIDTH, SLIDE_HEIGHT)
                .position_centered()
                .build()
                .unwrap();

            let mut canvas = window.into_canvas().build().unwrap();
            canvas.set_draw_color((0, 0, 0));
            canvas.clear();
            canvas.present();
            let mut event_pump = sdl_context.event_pump().unwrap();

            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);

            let texture_creator = canvas.texture_creator();
            let rendering_data = render::initialise_rendering_data(&state, &texture_creator);
            let mut slide_idx: usize = 0;

            let mut window_needs_redraw = true;

            for event in event_pump.wait_iter() {
                if window_needs_redraw {
                    let tick = std::time::Instant::now();
                    render::render(
                        &state,
                        &mut canvas,
                        slide_idx,
                        true,
                        &rendering_data,
                        args.rects,
                    );
                    let tock = std::time::Instant::now();
                    println!("rendered slide in {:6} us.", (tock - tick).as_micros());
                    window_needs_redraw = false;
                }

                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break,
                    Event::KeyDown {
                        keycode: Some(Keycode::Right),
                        ..
                    } => {
                        let new_idx = (number_of_slides - 1).min(slide_idx + 1);
                        if new_idx != slide_idx {
                            slide_idx = new_idx;
                            window_needs_redraw = true;
                        }
                    }
                    Event::KeyDown {
                        keycode: Some(Keycode::Left),
                        ..
                    } => {
                        let new_idx = slide_idx.saturating_sub(1);
                        if new_idx != slide_idx {
                            slide_idx = new_idx;
                            window_needs_redraw = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        FoliumSubcommand::Inspect { input } => {
            let state = ast::GlobalState::new();
            interpreter::load_from_file(&state, input).unwrap();
            println!("{state}");
        }
        FoliumSubcommand::ListFonts => {
            let mut database = fontdb::Database::new();
            database.load_system_fonts();
            let mut fonts = database
                .faces()
                .map(|f| f.families.first().unwrap().0.clone())
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            fonts.sort();
            println!("{}", fonts.join("\n"));
        }
    }
}
