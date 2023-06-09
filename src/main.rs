use cute::interpreter::lexer::{Lexer, Token};

fn main() {
    let s = include_str!("../test/_sqrt.cute");
    let mut l = Lexer::new(s.chars());
    loop {
        let token = l.next_token().unwrap();
        println!("{:?}", token);
        if let Token::EOF = token {
            break;
        }
    }
}
