use super::lexer::{Lexer, Token, LexerError};

struct Parser<'a> {
    lexer: Lexer<'a>
}

#[derive(Debug)]
enum LeftValue {
    Variable(String),
    Super(String),
    Field(String),
    Item
}

#[derive(Debug)]
struct BOp {
    priority: u8,
    name: &'static str
}

#[derive(Debug)]
enum ParserError {
    LexerError,
    UnexpectedToken,
    NotLeftValue
}

impl From<LexerError> for ParserError {
    fn from(_: LexerError) -> Self {
        Self::LexerError
    }
}

impl<'a> Parser<'a> {
    fn expect_single(&mut self, expected: char) -> Result<(), ParserError> {
        match self.lexer.next()? {
            Token::Single(char) if char == expected => Ok(()),
            _ => Err(ParserError::UnexpectedToken)
        }
    }

    fn expect_name(&mut self) -> Result<String, ParserError> {
        match self.lexer.next()? {
            Token::Name(name) => Ok(name),
            _ => Err(ParserError::UnexpectedToken)
        }
    }

    fn read_left_value(&mut self, lval: &LeftValue) {
        match lval {
            LeftValue::Variable(name) => println!("load {name}"),
            LeftValue::Super(name) => println!("load_super {name}"),
            LeftValue::Field(name) => println!("load_field {name}"),
            LeftValue::Item => println!("load_item")
        }
    }

    fn write_left_value(&mut self, lval: &LeftValue) {
        match lval {
            LeftValue::Variable(name) => println!("store {name}"),
            LeftValue::Super(name) => println!("store_super {name}"),
            LeftValue::Field(name) => println!("store_field {name}"),
            LeftValue::Item => println!("store_item")
        }
    }

    fn simple_expression(&mut self) -> Result<Option<LeftValue>, ParserError> {
        let mut lval = match self.lexer.next()? {
            Token::Name(name) => Some(LeftValue::Variable(name)),
            Token::Integer(value) => { println!("push_int {value}"); None },
            Token::Float(value) => { println!("push_float {value}"); None },
            Token::String(value) => { println!("push_string {value}"); None },
            Token::Single('$') => {
                let mut level = 0u8;
                while self.lexer.peek()? == &Token::Single('$') {
                    self.lexer.next()?;
                    level += 1;
                }
                let name = self.expect_name()?;
                if level == 0 {
                    Some(LeftValue::Super(name))
                } else {
                    println!("push_super {level}");
                    Some(LeftValue::Field(name))
                }
            }
            Token::Single('(') => {
                self.expression()?;
                self.expect_single(')')?;
                None
            }
            Token::Single('{') => {
                println!("push_closure =>");
                self.statement_list(&Token::Single('}'))?;
                println!("call 0");
                None
            }
            _ => return Err(ParserError::UnexpectedToken)
        };

        while let Token::Single('.' | '(' | '[') = self.lexer.peek()? {
            if let Some(lval) = lval {
                self.read_left_value(&lval);
            }

            lval = match self.lexer.next()? {
                Token::Single('.') => {
                    let name = self.expect_name()?;
                    Some(LeftValue::Field(name))
                }
                Token::Single('(') => {
                    // ... exp_list
                    self.expect_single(')')?;
                    println!("call <num>");
                    None
                }
                Token::Single('[') => {
                    self.expression()?;
                    self.expect_single(']')?;
                    Some(LeftValue::Item)
                }
                _ => None
            }
        }

        Ok(lval)
    }

    fn try_uop(&mut self) -> Result<Option<char>, ParserError> {
        let uop = match self.lexer.peek()? {
            Token::Single(char @ ('+' | '-' | '!' | '~' | '#')) => Some(*char),
            _ => None
        };

        Ok(uop)
    }

    fn try_bop(&mut self) -> Result<Option<BOp>, ParserError> {
        let bop = match self.lexer.peek()? {
            Token::Single(char) => match char {
                '+' => Some(BOp { priority: 10, name: "+" }),
                '-' => Some(BOp { priority: 10, name: "-" }),
                '*' => Some(BOp { priority: 11, name: "*" }),
                '/' => Some(BOp { priority: 11, name: "/" }),
                '%' => Some(BOp { priority: 11, name: "%" }),
                '>' => Some(BOp { priority: 3, name: ">" }),
                '<' => Some(BOp { priority: 3, name: "<" }),
                '|' => Some(BOp { priority: 4, name: "|" }),
                '^' => Some(BOp { priority: 5, name: "^" }),
                '&' => Some(BOp { priority: 6, name: "&" }),
                _ => None
            },
            Token::Eq => Some(BOp { priority: 3, name: "==" }),
            Token::Ne => Some(BOp { priority: 3, name: "!=" }),
            Token::Ge => Some(BOp { priority: 3, name: ">=" }),
            Token::Le => Some(BOp { priority: 3, name: "<=" }),
            Token::Shl => Some(BOp { priority: 7, name: "<<" }),
            Token::Shr => Some(BOp { priority: 7, name: ">>" }),
            Token::Ushr => Some(BOp { priority: 7, name: ">>>" }),
            Token::Or => Some(BOp { priority: 1, name: "||" }),
            Token::And => Some(BOp { priority: 2, name: "&&" }),
            _ => None
        };

        Ok(bop)
    }

    fn op_expression(&mut self, limit: u8) -> Result<Option<BOp>, ParserError> {
        let lval = if let Some(uop) = self.try_uop()? {
            self.lexer.next()?;
            self.op_expression(12)?;
            println!("uop {uop}");
            None
        } else {
            self.simple_expression()?
        };

        if let Token::Single('=') = self.lexer.peek()? {
            self.lexer.next()?;
            match lval {
                Some(lval) if limit == 0 => {
                    self.expression()?;
                    println!("dup");
                    self.write_left_value(&lval);
                    Ok(None)
                }
                _ => Err(ParserError::NotLeftValue)
            }
        } else {
            if let Some(lval) = lval {
                self.read_left_value(&lval);
            }

            let mut current_bop = self.try_bop()?;

            while let Some(bop) = &current_bop {
                if bop.priority <= limit {
                    break;
                }
                self.lexer.next()?;
                let next_bop = self.op_expression(bop.priority)?;
                println!("bop {}", bop.name);
                current_bop = next_bop;
            }

            Ok(current_bop)
        }
    }

    fn expression(&mut self) -> Result<(), ParserError> {
        match self.lexer.peek()? {
            Token::Single(':') => {
                self.lexer.next()?;
                println!("do {{");
                self.expression()?;
                println!("}} while nil");
            }
            Token::Single('<') => {
                self.lexer.next()?;
                self.expression()?;
                println!("return");
            }
            Token::Shl => {
                self.lexer.next()?;
                self.expression()?;
                println!("dup");
                println!("out");
            }
            _ => {
                self.op_expression(0)?;
            }
        }

        Ok(())
    }

    fn statement_list(&mut self, ending: &Token) -> Result<(), ParserError> {
        println!("{{");

        while self.lexer.peek()? != ending {
            self.expression()?;
            self.expect_single(';')?;
            println!("pop");
        }
        self.lexer.next()?;

        println!("push_self");
        println!("return");

        println!("}}");

        Ok(())
    }
}

pub fn parse(lexer: Lexer) {
    let mut parser = Parser { lexer };
    parser.statement_list(&Token::EOF).unwrap();
}
