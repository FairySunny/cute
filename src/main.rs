use cute::interpreter::lexer::{Lexer, Token};

fn main() {
    let s = include_str!("../test/_sqrt.cute");
    let mut l = Lexer::new(s.chars().peekable());
    loop {
        let token = l.next().unwrap();
        println!("{:?}", token);
        if let Token::EOF = token {
            break;
        }
    }
}
