#![allow(dead_code)]

use ast::GlobalState;

mod ast;
mod interpreter;

fn main() {
    let state = GlobalState::new();
    interpreter::load(&state, "test.flm").unwrap();
    // println!("{}", state);
}
