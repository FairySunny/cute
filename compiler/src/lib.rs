pub mod lexer;
pub mod parser;

use std::str::Chars;
use bytecode::program::{Program, ProgramBundle};
use lexer::Lexer;
use parser::ParserError;

pub fn compile_chars(chars: Chars) -> Result<ProgramBundle, ParserError> {
    let mut program = Program::new();
    parser::parse(Lexer::new(chars), &mut program)?;
    Ok(program.bundle())
}
