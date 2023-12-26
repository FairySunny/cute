pub mod lexer;
pub mod parser;

use std::{fs, path::Path, io};
use bytecode::program::{Program, ProgramBundle};
use lexer::Lexer;

pub fn load_file(path: impl AsRef<Path>) -> Result<ProgramBundle, io::Error> {
    let source = fs::read_to_string(path)?;
    let mut program = Program::new();
    parser::parse(Lexer::new(source.chars()), &mut program);
    Ok(program.bundle())
}
