use std::fmt::{Display, Formatter, Result};

#[derive(Clone, Debug, PartialEq)]
pub struct RatexToken {
    pub token: RatexTokenType,
    pub lexeme: String,
    pub line: u32,
}

impl Display for RatexToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use RatexTokenType as RXTT;

        match &self.token {
            RXTT::String(string) => {
                write!(f, "{} {} {}", self.token.to_string(), self.lexeme, string)
            }
            RXTT::Number(number) => {
                write!(f, "{} {} {}", self.token.to_string(), self.lexeme, number)
            }
            _ => {
                write!(f, "{} {}", self.token.to_string(), self.lexeme)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RatexTokenType {
    // Single-character tokens.
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,
    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    Identifier(String),
    String(String),
    Number(f64),
    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    EOF,
}

impl Display for RatexTokenType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}
