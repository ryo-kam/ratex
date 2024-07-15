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
    UnknownTokenError(u32, String),
    ScanOutOfRangeError(u32),
    UnterminatedString,
}

impl Display for RatexErrorType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            Self::UnknownTokenError(line, token) => {
                write!(f, "unknown token on line {}: {}", line, token)
            }
            Self::ScanOutOfRangeError(index) => {
                write!(f, "tried to scan invalid index: {}", index)
            }
            Self::UnterminatedString => {
                write!(f, "unterminated string")
            }
        }
    }
}

impl Error for RatexErrorType {}
