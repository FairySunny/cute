use crate::lexer::{Lexer, Token, LexerError};
use bytecode::{code, program::{Program, GeneratingError, JumpWhere}};

#[derive(Debug)]
pub enum ParserError {
    LexerError(LexerError),
    UnexpectedToken(Token),
    NotLeftValue,
    GeneratingError(GeneratingError)
}

impl From<LexerError> for ParserError {
    fn from(e: LexerError) -> Self {
        Self::LexerError(e)
    }
}

impl From<GeneratingError> for ParserError {
    fn from(e: GeneratingError) -> Self {
        Self::GeneratingError(e)
    }
}

struct Parser<'a, 'b> {
    lexer: Lexer<'a>,
    program: &'b mut Program
}

#[derive(Debug)]
enum LeftValue {
    Variable(String),
    Super(String),
    Field(String),
    Item,
    Slice
}

#[derive(Debug)]
enum UOpAction {
    NoOp,
    Loop,
    Arg,
    Return,
    In,
    Out,
    Code(u8)
}

#[derive(Debug)]
struct UOp {
    pri: u8,
    write_lval: bool,
    action: UOpAction
}

#[derive(Debug)]
enum BOpAction {
    Assign,
    Or,
    And,
    If,
    Code(u8)
}

#[derive(Debug)]
struct BOp {
    left_pri: u8,
    right_pri: u8,
    action: BOpAction
}

impl BOp {
    fn left(priority: u8, action: BOpAction) -> Self {
        Self {
            left_pri: priority,
            right_pri: priority,
            action
        }
    }

    fn left_c(priority: u8, code: u8) -> Self {
        Self::left(priority, BOpAction::Code(code))
    }

    fn right(priority: u8, action: BOpAction) -> Self {
        Self {
            left_pri: priority + 1,
            right_pri: priority,
            action
        }
    }
}

impl<'a, 'b> Parser<'a, 'b> {
    fn expect_single(&mut self, expected: char) -> Result<(), ParserError> {
        match self.lexer.next()? {
            Token::Single(char) if char == expected => Ok(()),
            token => Err(ParserError::UnexpectedToken(token))
        }
    }

    fn expect_name(&mut self) -> Result<String, ParserError> {
        match self.lexer.next()? {
            Token::Name(name) => Ok(name),
            token => Err(ParserError::UnexpectedToken(token))
        }
    }

    fn read_left_value(&mut self, lval: &LeftValue) -> Result<(), ParserError> {
        match lval {
            LeftValue::Variable(name) => {
                self.program.byte(code::LOAD);
                self.program.str(name)?;
            },
            LeftValue::Super(name) => {
                self.program.byte(code::LOAD_SUPER);
                self.program.str(name)?;
            },
            LeftValue::Field(name) => {
                self.program.byte(code::LOAD_FIELD);
                self.program.str(name)?;
            },
            LeftValue::Item => self.program.byte(code::LOAD_ITEM),
            LeftValue::Slice => self.program.byte(code::LOAD_SLICE)
        }
        Ok(())
    }

    fn write_left_value(&mut self, lval: &LeftValue) -> Result<(), ParserError> {
        match lval {
            LeftValue::Variable(name) => {
                self.program.byte(code::DUP);
                self.program.byte(code::STORE);
                self.program.str(name)?;
            },
            LeftValue::Super(name) => {
                self.program.byte(code::DUP);
                self.program.byte(code::STORE_SUPER);
                self.program.str(name)?;
            },
            LeftValue::Field(name) => {
                self.program.byte(code::DUP_PRE2);
                self.program.byte(code::STORE_FIELD);
                self.program.str(name)?;
            },
            LeftValue::Item => {
                self.program.byte(code::DUP_PRE3);
                self.program.byte(code::STORE_ITEM);
            },
            LeftValue::Slice => {
                self.program.byte(code::DUP_PRE4);
                self.program.byte(code::STORE_SLICE);
            }
        }
        Ok(())
    }

    fn simple_expression(&mut self) -> Result<Option<LeftValue>, ParserError> {
        let mut lval = match self.lexer.next()? {
            Token::Name(name) => Some(LeftValue::Variable(name)),
            Token::Integer(value) => { self.program.push_int(value)?; None },
            Token::Float(value) => { self.program.push_float(value)?; None },
            Token::String(value) => { self.program.push_str(&value)?; None },
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
                    self.program.byte(code::PUSH_SUPER);
                    self.program.byte(level);
                    Some(LeftValue::Field(name))
                }
            }
            Token::Single('(') => {
                self.expression()?;
                self.expect_single(')')?;
                None
            }
            Token::Single('{') => {
                self.program.push_closure_and_switch()?;
                self.statement_list(&Token::Single('}'))?;
                self.program.switch_back();
                self.program.byte(code::CALL);
                self.program.byte(0);
                None
            }
            Token::Single('@') => match self.lexer.next()? {
                Token::Single('{') => {
                    self.program.push_closure_and_switch()?;
                    self.statement_list(&Token::Single('}'))?;
                    self.program.switch_back();
                    None
                }
                Token::Name(name) | Token::String(name) => {
                    self.program.byte(code::LOAD_LIB);
                    self.program.str(&name)?;
                    None
                }
                token => return Err(ParserError::UnexpectedToken(token))
            }
            Token::Single('[') => {
                let cnt = self.expression_list(&Token::Single(']'))?;
                self.program.byte(code::NEW_ARRAY);
                self.program.byte(cnt);
                None
            }
            token => return Err(ParserError::UnexpectedToken(token))
        };

        while let &Token::Single(char @ ('.' | '(' | '[')) = self.lexer.peek()? {
            self.lexer.next()?;

            if let Some(lval) = &lval {
                self.read_left_value(lval)?;
            }

            lval = match char {
                '.' => {
                    let name = self.expect_name()?;
                    Some(LeftValue::Field(name))
                }
                '(' => {
                    let cnt = self.expression_list(&Token::Single(')'))?;
                    self.program.byte(code::CALL);
                    self.program.byte(cnt);
                    None
                }
                '[' => {
                    fn after_range(parser: &mut Parser) -> Result<Option<LeftValue>, ParserError> {
                        match parser.lexer.peek()? {
                            Token::Single(']') => {
                                parser.lexer.next()?;
                                parser.program.byte(code::PUSH_NULL);
                            }
                            _ => {
                                parser.expression()?;
                                parser.expect_single(']')?;
                            }
                        }
                        Ok(Some(LeftValue::Slice))
                    }

                    match self.lexer.peek()? {
                        Token::Single(':') => {
                            self.lexer.next()?;
                            self.program.byte(code::PUSH_NULL);
                            after_range(self)?
                        }
                        _ => {
                            self.expression()?;
                            match self.lexer.next()? {
                                Token::Single(':') => after_range(self)?,
                                Token::Single(']') => Some(LeftValue::Item),
                                token => return Err(ParserError::UnexpectedToken(token))
                            }
                        }
                    }
                }
                _ => None
            }
        }

        Ok(lval)
    }

    fn try_uop(&mut self) -> Result<Option<UOp>, ParserError> {
        let uop = match self.lexer.peek()? {
            Token::Single(char) => match char {
                '+' => Some(UOp { pri: 16, action: UOpAction::NoOp, write_lval: false }),
                '-' => Some(UOp { pri: 16, action: UOpAction::Code(code::NEG), write_lval: false }),
                '!' => Some(UOp { pri: 16, action: UOpAction::Code(code::NOT), write_lval: false }),
                '~' => Some(UOp { pri: 16, action: UOpAction::Code(code::BINV), write_lval: false }),
                '#' => Some(UOp { pri: 16, action: UOpAction::Code(code::LEN), write_lval: false }),
                '?' => Some(UOp { pri: 16, action: UOpAction::Code(code::TYPE), write_lval: false }),
                ':' => Some(UOp { pri: 0, action: UOpAction::Loop, write_lval: false }),
                '>' => Some(UOp { pri: 0, action: UOpAction::Arg, write_lval: true }),
                '<' => Some(UOp { pri: 0, action: UOpAction::Return, write_lval: false }),
                _ => None
            }
            Token::Shr => Some(UOp { pri: 0, action: UOpAction::In, write_lval: true }),
            Token::Shl => Some(UOp { pri: 0, action: UOpAction::Out, write_lval: false }),
            _ => None
        };

        Ok(uop)
    }

    fn try_bop(&mut self) -> Result<Option<BOp>, ParserError> {
        let bop = match self.lexer.peek()? {
            Token::Single(char) => match char {
                '+' => Some(BOp::left_c(14, code::ADD)),
                '-' => Some(BOp::left_c(14, code::SUB)),
                '*' => Some(BOp::left_c(15, code::MUL)),
                '/' => Some(BOp::left_c(15, code::DIV)),
                '%' => Some(BOp::left_c(15, code::MOD)),
                '>' => Some(BOp::left_c(7, code::CMP_GT)),
                '<' => Some(BOp::left_c(7, code::CMP_LT)),
                '|' => Some(BOp::left_c(8, code::BOR)),
                '^' => Some(BOp::left_c(9, code::BXOR)),
                '&' => Some(BOp::left_c(10, code::BAND)),
                '?' => Some(BOp::right(3, BOpAction::If)),
                '=' => Some(BOp::right(1, BOpAction::Assign)),
                _ => None
            },
            Token::Eq => Some(BOp::left_c(7, code::CMP_EQ)),
            Token::Ne => Some(BOp::left_c(7, code::CMP_NE)),
            Token::Ge => Some(BOp::left_c(7, code::CMP_GE)),
            Token::Le => Some(BOp::left_c(7, code::CMP_LE)),
            Token::Shl => Some(BOp::left_c(11, code::SHL)),
            Token::Shr => Some(BOp::left_c(11, code::SHR)),
            Token::Or => Some(BOp::left(4, BOpAction::Or)),
            Token::And => Some(BOp::left(5, BOpAction::And)),
            _ => None
        };

        Ok(bop)
    }

    fn op_expression(&mut self, limit: u8) -> Result<(Option<LeftValue>, Option<BOp>), ParserError> {
        let mut lval = match self.try_uop()? {
            Some(uop) => {
                self.lexer.next()?;

                let mut pos = 0usize;

                if let UOpAction::Loop = uop.action {
                    self.program.byte(code::PUSH_NULL);
                    pos = self.program.get_pos();
                    self.program.byte(code::POP);
                }

                let (lval, _) = self.op_expression(uop.pri)?;

                if !uop.write_lval {
                    if let Some(lval) = &lval {
                        self.read_left_value(lval)?;
                    }
                }

                match uop.action {
                    UOpAction::NoOp => {}
                    UOpAction::Loop => {
                        self.program.byte(code::DUP);
                        self.program.byte(code::JN);
                        self.program.jump_back(pos)?;
                    },
                    UOpAction::Arg => self.program.push_arg()?,
                    UOpAction::Return => self.program.byte(code::RETURN),
                    UOpAction::In => self.program.byte(code::IN),
                    UOpAction::Out => {
                        self.program.byte(code::DUP);
                        self.program.byte(code::OUT);
                    }
                    UOpAction::Code(code) => self.program.byte(code)
                }

                if uop.write_lval {
                    match &lval {
                        Some(lval) => self.write_left_value(&lval)?,
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

            let mut jump: Option<JumpWhere> = None;

            match bop.action {
                BOpAction::Assign => {}
                _ => if let Some(lval) = &lval {
                    self.read_left_value(lval)?;
                }
            }
            match bop.action {
                BOpAction::Or => {
                    self.program.byte(code::DUP);
                    self.program.byte(code::JT);
                    jump = Some(self.program.jump_where());
                    self.program.byte(code::POP);
                }
                BOpAction::And => {
                    self.program.byte(code::DUP);
                    self.program.byte(code::JF);
                    jump = Some(self.program.jump_where());
                    self.program.byte(code::POP);
                }
                BOpAction::If => {
                    self.program.byte(code::JF);
                    let jump1 = self.program.jump_where();
                    self.expression()?;
                    self.expect_single(':')?;
                    self.program.byte(code::JMP);
                    jump = Some(self.program.jump_where());
                    self.program.jump_here(jump1)?;
                }
                _ => {}
            }

            let (next_lval, next_bop) = self.op_expression(bop.right_pri)?;

            if let Some(lval) = &next_lval {
                self.read_left_value(lval)?;
            }

            match bop.action {
                BOpAction::Assign => match &lval {
                    Some(lval) => self.write_left_value(lval)?,
                    None => return Err(ParserError::NotLeftValue)
                }
                BOpAction::Or | BOpAction::And | BOpAction::If =>
                    self.program.jump_here(jump.unwrap())?,
                BOpAction::Code(code) =>
                    self.program.byte(code)
            }

            lval = None;
            current_bop = next_bop;
        }

        Ok((lval, current_bop))
    }

    fn expression(&mut self) -> Result<(), ParserError> {
        let (lval, _) = self.op_expression(0)?;

        if let Some(lval) = &lval {
            self.read_left_value(lval)?;
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
                token => return Err(ParserError::UnexpectedToken(token))
            }
        }

        Ok(cnt)
    }

    fn statement_list(&mut self, ending: &Token) -> Result<(), ParserError> {
        while self.lexer.peek()? != ending {
            self.expression()?;
            self.expect_single(';')?;
            self.program.byte(code::POP);
        }
        self.lexer.next()?;

        self.program.byte(code::PUSH_SELF);
        self.program.byte(code::RETURN);

        Ok(())
    }
}

pub fn parse(lexer: Lexer, program: &mut Program) -> Result<(), ParserError> {
    let mut parser = Parser { lexer, program };
    parser.statement_list(&Token::EOF)
}
