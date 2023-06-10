use super::lexer::{Lexer, Token, LexerError};

struct Parser<'a> {
    lexer: Lexer<'a>
}

#[derive(Debug)]
enum ParserError {
    LexerError,
    UnexpectedToken
}

impl From<LexerError> for ParserError {
    fn from(_: LexerError) -> Self {
        Self::LexerError
    }
}

impl<'a> Parser<'a> {
    fn expect(&mut self, expected: Token) -> Result<(), ParserError> {
        match self.lexer.next() {
            Err(_) => Err(ParserError::LexerError),
            Ok(token) if token == expected => Ok(()),
            _ => Err(ParserError::UnexpectedToken)
        }
    }

    fn simple_expression(&mut self) -> Result<(), ParserError> {
        match self.lexer.next()? {
            Token::Single('(') => {
                if !Self::is_expression(self.lexer.peek()?) {
                    return Err(ParserError::UnexpectedToken);
                }
                self.expression();
                self.expect(Token::Single(')'))?;
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
            _ => return Err(ParserError::UnexpectedToken)
        }

        Ok(())
    }

    fn is_expression(token: &Token) -> bool {
        todo!()
    }

    fn expression(&mut self) {
        todo!()
    }

    fn statement_list(&mut self) -> Result<(), ParserError> {
        println!("{{");

        while Self::is_expression(self.lexer.peek()?) {
            self.expression();
            self.expect(Token::Single(';'))?;
        }

        println!("}}");

        Ok(())
    }
}

pub fn parse(lexer: Lexer) {
    let mut parser = Parser { lexer };
    parser.statement_list().unwrap();
    parser.expect(Token::EOF).unwrap();
}
