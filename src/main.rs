#![allow(dead_code)]

mod ast;
mod error;
mod interpreter;
mod style;

fn main() {
    let state = ast::GlobalState::new();
    interpreter::load_from_file(&state, "test.flm").unwrap();
    println!("{}", state);
}
