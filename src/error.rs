use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result},
};

use crate::{ast::Object, token};

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
    VarInInitialiser,
    RedeclareLocalVariable(u32),

    // Interrupts
    Break,
    Return(Object),
}

impl Display for RatexErrorType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            RatexErrorType::UnknownToken(line, token) => {
                write!(f, "line {}, unknown token {}", line, token)
            }
            RatexErrorType::UnterminatedString(line, string) => {
                write!(f, "line {}, unterminated string: {}", line, string)
            }
            RatexErrorType::UnterminatedBlockComment(line, index) => {
                write!(f, "line {}, unterminated block comment: {}", line, index)
            }
            RatexErrorType::UnexpectedToken(line, token) => {
                write!(f, "line {}, unexpected token '{}'", line, token)
            }
            RatexErrorType::ExpectedToken(line, string) => {
                write!(
                    f,
                    "line {}, expected token '{}' but not found",
                    line, string
                )
            }
            RatexErrorType::UndefinedIdentifier(identifier) => {
                write!(f, "tried to read undefined variable '{}'", identifier)
            }
            RatexErrorType::InvalidAssignment(line) => {
                write!(f, "line {}, invalid assignment", line)
            }
            RatexErrorType::InvalidLogicalOperation(line) => {
                write!(f, "line {}, invalid logical operation", line)
            }
            RatexErrorType::InvalidFunctionCall => {
                write!(f, "invalid function call")
            }
            RatexErrorType::IncompatibleArity => {
                write!(f, "too many or too few arguments")
            }
            RatexErrorType::VarInInitialiser => {
                write!(f, "can't read local variable in its own initialiser")
            }
            RatexErrorType::Break => {
                write!(f, "break statement reached")
            }
            RatexErrorType::Return(_) => {
                write!(f, "returned")
            }
            RatexErrorType::RedeclareLocalVariable(line) => {
                write!(
                    f,
                    "line {}, there is already a variable with this name",
                    line
                )
            }
        }
    }
}

impl Error for RatexErrorType {}
