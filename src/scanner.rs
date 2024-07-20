use std::{collections::HashMap, fmt::format, iter::Peekable, str::Chars};

use crate::{
    error::{RatexError, RatexErrorType},
    token::{RatexToken, RatexTokenType},
};

pub struct Scanner<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    pub tokens: Vec<RatexToken>,
    start: usize,
    current: usize,
    line: u32,
    hash_map: HashMap<&'a str, RatexTokenType>,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        use RatexTokenType as RXTT;
        Scanner {
            source,
            chars: source.chars().peekable(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
            hash_map: HashMap::from([
                ("and", RXTT::And),
                ("class", RXTT::Class),
                ("else", RXTT::Else),
                ("false", RXTT::False),
                ("for", RXTT::For),
                ("fun", RXTT::Fun),
                ("if", RXTT::If),
                ("nil", RXTT::Nil),
                ("or", RXTT::Or),
                ("print", RXTT::Print),
                ("return", RXTT::Return),
                ("super", RXTT::Super),
                ("this", RXTT::This),
                ("true", RXTT::True),
                ("var", RXTT::Var),
                ("while", RXTT::While),
            ]),
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), RatexError> {
        while !self.is_at_end() {
            self.start = self.current;
            match self.scan_token() {
                Err(e) => {
                    println!("{e:?}")
                }
                _ => {}
            }
        }

        self.tokens.push(RatexToken {
            token: RatexTokenType::EOF,
            lexeme: "EOF".to_string(),
            line: self.line,
        });
        Ok(())
    }

    fn is_at_end(&self) -> bool {
        return self.current >= self.source.len();
    }

    fn scan_token(&mut self) -> Result<(), RatexError> {
        let c: char = self.advance().unwrap();
        let mut err: Option<RatexError> = None;

        use RatexTokenType as RXTT;

        match c {
            '(' => self.add_token(RXTT::LeftParen),
            ')' => self.add_token(RXTT::RightParen),
            '{' => self.add_token(RXTT::LeftBrace),
            '}' => self.add_token(RXTT::RightBrace),
            ',' => self.add_token(RXTT::Comma),
            '.' => self.add_token(RXTT::Dot),
            '-' => self.add_token(RXTT::Minus),
            '+' => self.add_token(RXTT::Plus),
            ';' => self.add_token(RXTT::Semicolon),
            '*' => self.add_token(RXTT::Star),
            '!' => {
                if self.advance_if('=') {
                    self.add_token(RXTT::BangEqual)
                } else {
                    self.add_token(RXTT::Bang)
                }
            }
            '=' => {
                if self.advance_if('=') {
                    self.add_token(RXTT::EqualEqual)
                } else {
                    self.add_token(RXTT::Equal)
                }
            }
            '>' => {
                if self.advance_if('=') {
                    self.add_token(RXTT::GreaterEqual)
                } else {
                    self.add_token(RXTT::Greater)
                }
            }
            '<' => {
                if self.advance_if('=') {
                    self.add_token(RXTT::LessEqual)
                } else {
                    self.add_token(RXTT::Less)
                }
            }
            '/' => {
                if self.advance_if('/') {
                    while !self.is_at_end() && *self.chars.peek().unwrap() != '\n' {
                        self.advance().unwrap();
                    }
                } else if self.advance_if('*') {
                    let mut terminated = false;

                    while !self.is_at_end() {
                        if self.advance_if('*') {
                            if self.advance_if('/') {
                                terminated = true;
                                break;
                            }
                        } else {
                            self.advance().unwrap();
                        }
                    }

                    let value = self
                        .source
                        .get(self.start..self.current)
                        .unwrap()
                        .to_owned();

                    if !terminated {
                        return Err(RatexError {
                            source: RatexErrorType::UnterminatedBlockComment(value),
                        });
                    }
                } else {
                    self.add_token(RXTT::Slash)
                }
            }
            '"' => match self.scan_string() {
                Err(e) => err = Some(e),
                _ => {}
            },
            '0'..='9' => self.scan_number()?,
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            'a'..='z' | 'A'..='Z' | '_' => {
                self.scan_identifier()?;
            }
            _ => {
                err = Some(RatexError {
                    source: RatexErrorType::UnknownToken(self.line, c.to_string()),
                })
            }
        }

        match err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    fn advance(&mut self) -> Option<char> {
        self.current += 1;

        self.chars.next()
    }

    fn add_token(&mut self, token: RatexTokenType) {
        let text = format!("{}", token.to_string());

        self.tokens.push(RatexToken {
            token,
            lexeme: text,
            line: self.line,
        });
    }

    fn advance_if(&mut self, next_char: char) -> bool {
        match self.chars.peek() {
            Some(char) => {
                if *char == next_char {
                    self.chars.next();
                    self.current += 1;
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    fn scan_string(&mut self) -> Result<(), RatexError> {
        while !self.is_at_end() && *self.chars.peek().unwrap() != '"' {
            if *self.chars.peek().unwrap() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(RatexError {
                source: RatexErrorType::UnterminatedString(
                    self.source
                        .get(self.start..self.current - 1)
                        .unwrap()
                        .to_owned(),
                ),
            });
        }

        self.advance();

        let value = self
            .source
            .get(self.start + 1..self.current - 1)
            .unwrap()
            .to_owned();

        self.add_token(RatexTokenType::String(value));
        Ok(())
    }

    fn scan_number(&mut self) -> Result<(), RatexError> {
        while !self.is_at_end() && self.chars.peek().unwrap().is_digit(10) {
            self.advance();
        }

        if !self.is_at_end() && *self.chars.peek().unwrap() == '.' {
            self.advance();

            // check if there's a number after the period to make sure it's a decimal point
            if !self.is_at_end() && self.chars.peek().unwrap().is_digit(10) {
                while !self.is_at_end() && self.chars.peek().unwrap().is_digit(10) {
                    self.advance();
                }
            } else {
                self.add_token(RatexTokenType::Number(
                    self.source
                        .get(self.start..self.current - 1)
                        .unwrap()
                        .parse::<f64>()
                        .unwrap(),
                ));
                self.add_token(RatexTokenType::Dot);
                return Ok(());
            }
        }

        self.add_token(RatexTokenType::Number(
            self.source
                .get(self.start..self.current)
                .unwrap()
                .parse::<f64>()
                .unwrap(),
        ));

        Ok(())
    }

    fn scan_identifier(&mut self) -> Result<(), RatexError> {
        while !self.is_at_end() && self.chars.peek().unwrap().is_alphanumeric() {
            self.advance();
        }

        let value = self.source.get(self.start..self.current).unwrap();

        let token_type = match self.hash_map.get(value) {
            Some(token) => token.clone(),
            None => RatexTokenType::Identifier(value.to_owned()),
        };

        self.add_token(token_type);

        Ok(())
    }
}
