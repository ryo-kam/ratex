use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result},
};

#[derive(Debug)]
pub struct RatexError {
    pub source: RatexErrorType,
}

impl Display for RatexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.source)
    }
}

impl Error for RatexError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

#[derive(Debug)]
pub enum RatexErrorType {
    UnknownToken(u32, String),
    UnterminatedString(String),
    UnterminatedBlockComment(String),
    UnterminatedParen(u32),
    UnexpectedToken(u32, String),
}

impl Display for RatexErrorType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::UnknownToken(line, token) => {
                write!(f, "unknown token on line {}: {}", line, token)
            }
            Self::UnterminatedBlockComment(index) => {
                write!(f, "unterminated block comment: {}", index)
            }
            Self::UnterminatedString(string) => {
                write!(f, "unterminated string: {}", string)
            }
            Self::UnexpectedToken(line, token) => {
                write!(f, "unexpected token on line {}: {}", line, token)
            }
            Self::UnterminatedParen(line) => {
                write!(f, "unclosed parenthesis on line {}", line)
            }
        }
    }
}

impl Error for RatexErrorType {}
