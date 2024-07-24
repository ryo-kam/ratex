use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result},
};

use crate::ast::Object;

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
    UnterminatedString(u32, String),
    UnterminatedBlockComment(u32, String),
    UnexpectedToken(u32, String),
    ExpectedToken(u32, String),
    UndefinedIdentifier(String),
    InvalidAssignment(u32),
    InvalidLogicalOperation(u32),
    InvalidFunctionCall,
    IncompatibleArity,
    Break,
    Return(Object),
}

impl Display for RatexErrorType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::UnknownToken(line, token) => {
                write!(f, "line {}, unknown token {}", line, token)
            }
            Self::UnterminatedBlockComment(line, index) => {
                write!(f, "line {}, unterminated block comment: {}", line, index)
            }
            Self::UnterminatedString(line, string) => {
                write!(f, "line {}, unterminated string: {}", line, string)
            }
            Self::UnexpectedToken(line, token) => {
                write!(f, "line {}, unexpected token '{}'", line, token)
            }
            Self::ExpectedToken(line, string) => {
                write!(
                    f,
                    "line {}, expected token '{}' but not found",
                    line, string
                )
            }
            Self::UndefinedIdentifier(identifier) => {
                write!(f, "tried to read undefined variable '{}'", identifier)
            }
            Self::InvalidAssignment(line) => {
                write!(f, "line {}, invalid assignment", line)
            }
            Self::InvalidLogicalOperation(line) => {
                write!(f, "line {}, invalid logical operation", line)
            }
            Self::Break => {
                write!(f, "break statement reached")
            }
            Self::InvalidFunctionCall => {
                write!(f, "invalid function call")
            }
            Self::IncompatibleArity => {
                write!(f, "too many or too few arguments")
            }
            Self::Return(_) => {
                write!(f, "returned")
            }
        }
    }
}

impl Error for RatexErrorType {}
