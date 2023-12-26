pub mod lexer;
pub mod parser;

use std::{str::Chars, fs, path::Path, io};
use bytecode::program::{Program, ProgramBundle};
use lexer::Lexer;

pub fn compile_chars(chars: Chars) -> ProgramBundle {
    let mut program = Program::new();
    parser::parse(Lexer::new(chars), &mut program);
    program.bundle()
}

pub fn compile_file(path: impl AsRef<Path>) -> Result<ProgramBundle, io::Error> {
    Ok(compile_chars(fs::read_to_string(path)?.chars()))
}
