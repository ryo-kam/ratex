use crate::token::RatexToken;
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result},
};

macro_rules! runtime_error_derive {
    (
        $error_name: ident ($prop: ident : $type: ty),
        $($variant_name: ident ($($message:expr),*)),+
    ) => {
        #[derive(Debug)]
        pub enum $error_name {
            $(
                $variant_name($type)
            ),+
        }

        impl Display for $error_name {
            fn fmt(&self, f: &mut Formatter) -> Result {
                match self {
                    $(
                        $error_name::$variant_name($prop) => {
                            write!(f, $($message),*);
                            write!(f, "--> {}:{}", $prop.line, $prop.position)
                        }
                    ),+
                }
            }
        }

        impl Error for $error_name {}
    };
}

runtime_error_derive! {
    // Errors
    RatexErrorType(token: RatexToken),
    UnknownToken("Unknown Token"),
    UnterminatedString("Unterminated String"),
    UnterminatedBlockComment("Unterminated Block Comment"),
    UnexpectedToken("Unexpected token: {}", token.lexeme),
    ExpectedToken("Expected Token: {}", token.lexeme),
    UndefinedIdentifier(""),
    InvalidAssignment(""),
    InvalidLogicalOperation(""),
    InvalidFunctionCall(""),
    IncompatibleArity(""),
    VarInInitialiser(""),
    RedeclareLocalVariable(""),
    InvalidReturnLocation(""),
    AccessUnknownField(""),
    NonInstanceSet("")
}
