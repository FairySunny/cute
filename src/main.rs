use bytecode::program::Program;
use compiler::{lexer::Lexer, parser};

fn main() {
    let s = include_str!("../test/_add.cute");
    let mut p = Program::new();
    parser::parse(Lexer::new(s.chars()), &mut p);
}
