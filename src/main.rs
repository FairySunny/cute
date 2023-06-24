use compiler::{lexer::Lexer, parser};

fn main() {
    let s = include_str!("../test/_add.cute");
    parser::parse(Lexer::new(s.chars()));
}
