#![allow(dead_code)]

mod ast;
mod error;
mod interpreter;
mod layout;
mod style;

fn main() {
    let state = ast::GlobalState::new();
    interpreter::load_from_file(&state, "test.flm").unwrap();
    println!("{}", state);

    let slides = state.slides.borrow();
    let slide = &slides[0];

    let layouted_elems = slide.layout(&state);

    dbg!(layouted_elems);

    
}
