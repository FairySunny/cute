use cute::interpreter::{lexer::Lexer, parser};

fn main() {
    let s = include_str!("../test/_sqrt.cute");
    parser::parse(Lexer::new(s.chars()));
}
