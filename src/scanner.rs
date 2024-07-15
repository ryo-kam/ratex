use std::{fmt::Error, iter::Peekable, str::Chars};

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
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source,
            chars: source.chars().peekable(),
            tokens: vec![],
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), RatexError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?
        }

        self.tokens.push(RatexToken {
            token: RatexTokenType::EOF,
            lexeme: "".to_string(),
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
                } else {
                    self.add_token(RXTT::Slash)
                }
            }
            '"' => match self.scan_string() {
                Err(e) => err = Some(e),
                _ => {}
            },
            '0'..'9' => self.scan_number()?,
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            _ => {
                err = Some(RatexError {
                    source: RatexErrorType::UnknownTokenError(self.line, c.to_string()),
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

        match self.chars.next() {
            Some(char) => Some(char),
            None => None,
        }
    }

    fn add_token(&mut self, token: RatexTokenType) {
        let text = "".to_owned();

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
                source: RatexErrorType::UnterminatedString,
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

            while !self.is_at_end() && self.chars.peek().unwrap().is_digit(10) {
                self.advance();
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
}
