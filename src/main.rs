#![allow(dead_code)]

mod ast;
mod error;
mod interpreter;
mod layout;
mod style;

use std::num::NonZeroU32;
use std::rc::Rc;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::layout::Rect;

fn main() {
    let state = ast::GlobalState::new();
    interpreter::load_from_file(&state, "test.flm").unwrap();
    println!("{}", state);

    let event_loop = EventLoop::new().unwrap();
    let window = Rc::new(
        WindowBuilder::new()
            .with_maximized(true)
            .with_inner_size(LogicalSize::new(1920.0, 1080.0))
            .build(&event_loop)
            .unwrap(),
    );
    let context = softbuffer::Context::new(window.clone()).unwrap();
    let mut surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Wait);

            match event {
                Event::WindowEvent {
                    window_id,
                    event: WindowEvent::RedrawRequested,
                } if window_id == window.id() => {
                    let (width, height) = {
                        let size = window.inner_size();
                        (size.width, size.height)
                    };
                    surface
                        .resize(
                            NonZeroU32::new(width).unwrap(),
                            NonZeroU32::new(height).unwrap(),
                        )
                        .unwrap();

                    let slides = state.slides.borrow();
                    let slide = &slides[0];

                    let layouted_elems = slide.layout(
                        &state,
                        Some(Rect {
                            x: 0,
                            y: 0,
                            w: width,
                            h: height,
                        }),
                    );

                    dbg!(&layouted_elems);

                    for lem in &layouted_elems {
                        println!(
                            "{}, {}",
                            lem.max_bounds.x + lem.max_bounds.w,
                            lem.max_bounds.y + lem.max_bounds.h
                        );
                    }

                    let mut buffer = surface.buffer_mut().unwrap();

                    for y in 0..height {
                        for x in 0..width {
                            let idx = x % width + y * width;
                            buffer[idx as usize] = 0x00ff00;
                        }
                    }

                    for layout_elem in &layouted_elems {
                        let rect = layout_elem.max_bounds;
                        for y in rect.y..rect.y + rect.h {
                            for x in rect.x..rect.x + rect.w {
                                let idx = x % width + y * width;
                                buffer[idx as usize] = 0xff0000;
                            }
                        }
                    }

                    buffer.present().unwrap();
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    window_id,
                } if window_id == window.id() => {
                    elwt.exit();
                }
                _ => {}
            }
        })
        .unwrap();
}
