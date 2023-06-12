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
struct UOp {
    pri: u8,
    write_lval: bool,
    name: &'static str
}

#[derive(Debug)]
struct BOp {
    left_pri: u8,
    right_pri: u8,
    name: &'static str
}

impl BOp {
    fn left(priority: u8, name: &'static str) -> Self {
        Self {
            left_pri: priority,
            right_pri: priority,
            name
        }
    }

    fn right(priority: u8, name: &'static str) -> Self {
        Self {
            left_pri: priority + 1,
            right_pri: priority,
            name
        }
    }
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
            Token::Single('@') => match self.lexer.next()? {
                Token::Single('{') => {
                    println!("push_closure =>");
                    self.statement_list(&Token::Single('}'))?;
                    None
                }
                Token::Name(name) => {
                    println!("load_lib {name}");
                    None
                }
                _ => return Err(ParserError::UnexpectedToken)
            }
            Token::Single('[') => {
                let cnt = self.expression_list(&Token::Single(']'))?;
                println!("new_array {cnt}");
                None
            }
            _ => return Err(ParserError::UnexpectedToken)
        };

        while let &Token::Single(char @ ('.' | '(' | '[')) = self.lexer.peek()? {
            self.lexer.next()?;

            if let Some(lval) = &lval {
                self.read_left_value(lval);
            }

            lval = match char {
                '.' => {
                    let name = self.expect_name()?;
                    Some(LeftValue::Field(name))
                }
                '(' => {
                    let cnt = self.expression_list(&Token::Single(')'))?;
                    println!("call {cnt}");
                    None
                }
                '[' => {
                    self.expression()?;
                    self.expect_single(']')?;
                    Some(LeftValue::Item)
                }
                _ => None
            }
        }

        Ok(lval)
    }

    fn try_uop(&mut self) -> Result<Option<UOp>, ParserError> {
        let uop = match self.lexer.peek()? {
            Token::Single(char) => match char {
                '+' => Some(UOp { pri: 16, name: "+", write_lval: false }),
                '-' => Some(UOp { pri: 16, name: "-", write_lval: false }),
                '!' => Some(UOp { pri: 16, name: "!", write_lval: false }),
                '~' => Some(UOp { pri: 16, name: "~", write_lval: false }),
                '#' => Some(UOp { pri: 16, name: "#", write_lval: false }),
                ':' => Some(UOp { pri: 0, name: ":", write_lval: false }),
                '>' => Some(UOp { pri: 0, name: ">", write_lval: true }),
                '<' => Some(UOp { pri: 0, name: "<", write_lval: false }),
                _ => None
            }
            Token::Shr => Some(UOp { pri: 0, name: ">>", write_lval: true }),
            Token::Shl => Some(UOp { pri: 0, name: "<<", write_lval: false }),
            _ => None
        };

        Ok(uop)
    }

    fn try_bop(&mut self) -> Result<Option<BOp>, ParserError> {
        let bop = match self.lexer.peek()? {
            Token::Single(char) => match char {
                '+' => Some(BOp::left(14, "+")),
                '-' => Some(BOp::left(14, "-")),
                '*' => Some(BOp::left(15, "*")),
                '/' => Some(BOp::left(15, "/")),
                '%' => Some(BOp::left(15, "%")),
                '>' => Some(BOp::left(7, ">")),
                '<' => Some(BOp::left(7, "<")),
                '|' => Some(BOp::left(8, "|")),
                '^' => Some(BOp::left(9, "^")),
                '&' => Some(BOp::left(10, "&")),
                '?' => Some(BOp::right(3, "?:")),
                '=' => Some(BOp::right(1, "=")),
                _ => None
            },
            Token::Eq => Some(BOp::left(7, "==")),
            Token::Ne => Some(BOp::left(7, "!=")),
            Token::Ge => Some(BOp::left(7, ">=")),
            Token::Le => Some(BOp::left(7, "<=")),
            Token::Shl => Some(BOp::left(11, "<<")),
            Token::Shr => Some(BOp::left(11, ">>")),
            Token::Ushr => Some(BOp::left(11, ">>>")),
            Token::Or => Some(BOp::left(5, "||")),
            Token::And => Some(BOp::left(6, "&&")),
            _ => None
        };

        Ok(bop)
    }

    fn op_expression(&mut self, limit: u8) -> Result<(Option<LeftValue>, Option<BOp>), ParserError> {
        let mut lval = match self.try_uop()? {
            Some(uop) => {
                self.lexer.next()?;

                if uop.name == ":" {
                    println!("do {{");
                }

                let (lval, _) = self.op_expression(uop.pri)?;

                match uop.name {
                    ":" => println!("}} while nil"),
                    ">" => println!("push_arg <idx>"),
                    "<" => println!("return"),
                    ">>" => println!("in"),
                    "<<" => {
                        println!("dup");
                        println!("out");
                    }
                    name => println!("uop {name}")
                }

                if uop.write_lval {
                    match &lval {
                        Some(lval) => {
                            println!("dup");
                            self.write_left_value(&lval);
                        }
                        None => return Err(ParserError::NotLeftValue)
                    }
                }

                None
            }
            None => self.simple_expression()?
        };

        let mut current_bop = self.try_bop()?;

        while let Some(bop) = &current_bop {
            if bop.left_pri <= limit {
                break;
            }
            self.lexer.next()?;

            if bop.name != "=" {
                if let Some(lval) = &lval {
                    self.read_left_value(lval);
                }
            }
            match bop.name {
                "||" => {
                    println!("dup");
                    println!("if false {{");
                    println!("pop");
                }
                "&&" => {
                    println!("dup");
                    println!("if true {{");
                    println!("pop");
                }
                "?:" => {
                    println!("if true {{");
                    self.expression()?;
                    self.expect_single(':')?;
                    println!("}} else {{");
                }
                _ => {}
            }

            let (next_lval, next_bop) = self.op_expression(bop.right_pri)?;

            if let Some(lval) = &next_lval {
                self.read_left_value(lval);
            }

            match bop.name {
                "=" => match &lval {
                    Some(lval) => {
                        println!("dup");
                        self.write_left_value(lval);
                    }
                    None => return Err(ParserError::NotLeftValue)
                }
                "||" | "&&" | "?:" => println!("}}"),
                _ => println!("bop {}", bop.name)
            }

            lval = None;
            current_bop = next_bop;
        }

        Ok((lval, current_bop))
    }

    fn expression(&mut self) -> Result<(), ParserError> {
        let (lval, _) = self.op_expression(0)?;

        if let Some(lval) = &lval {
            self.read_left_value(lval);
        }

        Ok(())
    }

    fn expression_list(&mut self, ending: &Token) -> Result<u8, ParserError> {
        if self.lexer.peek()? == ending {
            self.lexer.next()?;
            return Ok(0);
        }

        let mut cnt = 0u8;

        loop {
            self.expression()?;
            cnt += 1;

            match self.lexer.next()? {
                Token::Single(',') => {},
                token if &token == ending => break,
                _ => return Err(ParserError::UnexpectedToken)
            }
        }

        Ok(cnt)
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
