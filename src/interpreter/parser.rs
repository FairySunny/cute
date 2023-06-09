use super::lexer::{Lexer, Token, LexerError};

struct Parser<'a> {
    lexer: Lexer<'a>
}

#[derive(Debug)]
enum ParserError {
    LexerError,
    UnexpectedToken(Token)
}

impl From<LexerError> for ParserError {
    fn from(_: LexerError) -> Self {
        Self::LexerError
    }
}

impl<'a> Parser<'a> {
    fn expect(&mut self, expected: Token) -> Result<(), ParserError> {
        match self.lexer.next_token() {
            Err(_) => Err(ParserError::LexerError),
            Ok(token) if token == expected => Ok(()),
            Ok(token) => Err(ParserError::UnexpectedToken(token))
        }
    }

    fn simple_expression(&mut self, token: Token) -> Result<(), ParserError> {
        match token {
            Token::Single('(') => {
                let token = self.lexer.next_token()?;

                if Self::is_expression(&token) {
                    self.expression(token);
                    self.expect(Token::Single(')'))?;
                } else {
                    return Err(ParserError::UnexpectedToken(token));
                }
            }
            Token::Single('$') => {
                todo!()
            }
            Token::Integer(value) => println!("push int {value}"),
            Token::Float(value) => println!("push float {value}"),
            Token::String(value) => println!("push string {value}"),
            Token::Name(name) => {
                todo!()
            }
            _ => return Err(ParserError::UnexpectedToken(token))
        }

        Ok(())
    }

    fn is_expression(token: &Token) -> bool {
        todo!()
    }

    fn expression(&mut self, token: Token) {
        todo!()
    }

    fn statement_list(&mut self, in_closure: bool) -> Result<(), ParserError> {
        println!("{{");

        loop {
            let token = self.lexer.next_token()?;

            if Self::is_expression(&token) {
                self.expression(token);
                self.expect(Token::Single(';'))?;
            } else {
                match token {
                    Token::Single('}') if in_closure => {}
                    Token::EOF if !in_closure => {}
                    _ => return Err(ParserError::UnexpectedToken(token))
                }
                break;
            }
        }

        println!("}}");

        Ok(())
    }
}

pub fn parse(lexer: Lexer) {
    let mut parser = Parser { lexer };
    parser.statement_list(false).unwrap();
}
