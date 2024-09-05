pub struct Lexer<'a> {
    input: std::iter::Peekable<std::str::Chars<'a>>,
    peeked: Option<Token>
}

#[derive(PartialEq, Debug)]
pub enum Token {
    Eq, Ne, Ge, Le, And, Or,
    Shl, Shr,
    Single(char),
    Integer(i64),
    Float(f64),
    String(String),
    Name(String),
    EOF
}

#[derive(Debug)]
pub enum LexerError {
    CharacterError(char),
    MultiCommentError,
    NumberError,
    StringError
}

impl<'a> Lexer<'a> {
    fn skip_line(&mut self) {
        if let Some('\r') = self.input.find(|&char| char == '\r' || char == '\n') {
            self.input.next_if_eq(&'\n');
        }
    }

    fn skip_multi_comment(&mut self) -> Result<(), LexerError> {
        loop {
            if let None = self.input.find(|&char| char == '*') {
                return Err(LexerError::MultiCommentError);
            }

            if let Some('/') = self.input.next() {
                return Ok(());
            }
        }
    }

    fn check_double(&mut self, first: char, second: char, double: Token) -> Token {
        if let Some(&char) = self.input.peek() {
            if char == second {
                self.input.next();
                return double;
            }
        }

        Token::Single(first)
    }

    fn read_digits(&mut self, str: &mut String, radix: u32) {
        loop {
            match self.input.next_if(|char| char.is_digit(radix)) {
                Some(char) => str.push(char),
                None => break
            }
        }
    }

    fn parse_integer(&mut self, radix: u32) -> Result<Token, LexerError> {
        let mut str = String::new();

        self.read_digits(&mut str, radix);

        i64::from_str_radix(&str, radix)
            .map_or(Err(LexerError::NumberError), |num| Ok(Token::Integer(num)))
    }

    fn parse_float(&mut self, first: char) -> Result<Token, LexerError> {
        let mut str = String::from(first);

        let mut is_integer = true;

        if first == '.' {
            is_integer = false;

            self.read_digits(&mut str, 10);

            if str.len() == 1 {
                return Ok(Token::Single('.'));
            }
        } else {
            if first != '0' {
                self.read_digits(&mut str, 10);
            }

            if let Some(_) = self.input.next_if_eq(&'.') {
                is_integer = false;

                str.push('.');

                self.read_digits(&mut str, 10);
            }
        }

        if let Some(char) = self.input.next_if(|&char| char == 'E' || char == 'e') {
            is_integer = false;

            str.push(char);

            if let Some(char) = self.input.next_if(|&char| char == '+' || char == '-') {
                str.push(char);
            }

            match self.input.next() {
                Some('0') => str.push('0'),
                Some(char @ '1'..='9') => {
                    str.push(char);
                    self.read_digits(&mut str, 10);
                }
                _ => return Err(LexerError::NumberError)
            }
        }

        if is_integer {
            i64::from_str_radix(&str, 10)
                .map_or(Err(LexerError::NumberError), |num| Ok(Token::Integer(num)))
        } else {
            str.parse()
                .map_or(Err(LexerError::NumberError), |num| Ok(Token::Float(num)))
        }
    }

    fn parse_number(&mut self, first: char) -> Result<Token, LexerError> {
        match first {
            '0' => match self.input.peek() {
                Some('0'..='7') => self.parse_integer(8),
                Some('X' | 'x') => {
                    self.input.next();
                    self.parse_integer(16)
                },
                Some('B' | 'b') => {
                    self.input.next();
                    self.parse_integer(2)
                },
                Some('.' | 'E' | 'e') => self.parse_float(first),
                _ => Ok(Token::Integer(0))
            }
            _ => self.parse_float(first)
        }
    }

    fn parse_string(&mut self, first: char) -> Result<Token, LexerError> {
        let mut str = String::new();

        loop {
            let char = match self.input.next() {
                Some('\r' | '\n') | None => return Err(LexerError::StringError),
                Some('\\') => match self.input.next() {
                    Some('r') => '\r',
                    Some('n') => '\n',
                    Some(char @ ('"' | '\'' | '\\')) => char,
                    _ => return Err(LexerError::StringError)
                }
                Some(char) => {
                    if char == first {
                        break;
                    }
                    char
                }
            };

            str.push(char);
        }

        Ok(Token::String(str))
    }

    fn parse_name(&mut self, first: char) -> Token {
        let mut name = String::from(first);

        loop {
            match self.input.next_if(|char|
                ('A'..='Z').contains(char) ||
                ('a'..='z').contains(char) ||
                ('0'..='9').contains(char) ||
                *char == '_'
            ) {
                Some(char) => name.push(char),
                None => break
            }
        }

        Token::Name(name)
    }

    fn parse_token(&mut self) -> Result<Token, LexerError> {
        loop {
            let char = match self.input.next() {
                Some(char) => char,
                None => return Ok(Token::EOF)
            };

            let token = match char {
                '#' => match self.input.peek() {
                    Some('!') => {
                        self.skip_line();
                        continue;
                    }
                    _ => Token::Single('#')
                }
                '/' => match self.input.peek() {
                    Some('/') => {
                        self.skip_line();
                        continue;
                    }
                    Some('*') => {
                        self.skip_multi_comment()?;
                        continue;
                    }
                    _ => Token::Single('/')
                }
                ' ' | '\t' | '\r' | '\n' => continue,
                '=' => self.check_double('=', '=', Token::Eq),
                '!' => self.check_double('!', '=', Token::Ne),
                '&' => self.check_double('&', '&', Token::And),
                '|' => self.check_double('|', '|', Token::Or),
                '>' => match self.input.peek() {
                    Some('=') => {
                        self.input.next();
                        Token::Ge
                    }
                    Some('>') => {
                        self.input.next();
                        Token::Shr
                    }
                    _ => Token::Single('>')
                }
                '<' => match self.input.peek() {
                    Some('=') => {
                        self.input.next();
                        Token::Le
                    }
                    Some('<') => {
                        self.input.next();
                        Token::Shl
                    }
                    _ => Token::Single('<')
                }
                '(' | ')' | '{' | '}' | '[' | ']' | ';' | ',' | '?' | ':' |
                '+' | '-' | '*' | '%' | '^' | '~' | '@' | '$'
                    => Token::Single(char),
                '0'..='9' | '.' => self.parse_number(char)?,
                '"' | '\'' => self.parse_string(char)?,
                'A'..='Z' | 'a'..='z' | '_' => self.parse_name(char),
                char => return Err(LexerError::CharacterError(char))
            };

            return Ok(token);
        }
    }

    pub fn new(input: std::str::Chars<'a>) -> Self {
        Self {
            input: input.peekable(),
            peeked: None
        }
    }

    pub fn next(&mut self) -> Result<Token, LexerError> {
        match self.peeked.take() {
            Some(token) => Ok(token),
            None => self.parse_token()
        }
    }

    pub fn peek(&mut self) -> Result<&Token, LexerError> {
        if let None = self.peeked {
            self.peeked = Some(self.parse_token()?);
        }

        Ok(self.peeked.as_ref().unwrap())
    }
}
