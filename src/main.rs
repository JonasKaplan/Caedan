#![allow(clippy::needless_return)]
#![allow(clippy::unused_unit)]

mod procedure;
mod region;
mod program;
mod parser;

use std::path::PathBuf;

use program::Program;

fn main() {
    let program: Program = Program::from_source(&PathBuf::from("examples/back_reference.cae")).unwrap();
    program.run();
}
