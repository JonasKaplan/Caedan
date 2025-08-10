#![allow(clippy::needless_return)]
#![allow(clippy::unused_unit)]

mod procedure;
mod region;
mod interpreter;
mod parser;

use std::path::PathBuf;

use interpreter::program::Program;

fn main() {
    let program: Program = Program::from_source(&PathBuf::from("examples/math.cae")).unwrap();
    program.run();
}
