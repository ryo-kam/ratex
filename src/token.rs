use std::{
    fmt::{Display, Formatter, Result},
    hash::Hash,
};

#[derive(Clone, Debug, PartialEq, Default, Eq)]
pub struct RatexToken {
    pub token_type: RatexTokenType,
    pub lexeme: String,
    pub line: u32,
    pub position: u32,
}

impl Display for RatexToken {
    fn fmt(&self, f: &mut Formatter) -> Result {
        use RatexTokenType as RXTT;

        match &self.token_type {
            RXTT::String(string) => {
                write!(
                    f,
                    "{} {} {}",
                    self.token_type.to_string(),
                    self.lexeme,
                    string
                )
            }
            RXTT::Number(number) => {
                write!(
                    f,
                    "{} {} {}",
                    self.token_type.to_string(),
                    self.lexeme,
                    number
                )
            }
            _ => {
                write!(f, "{} {}", self.token_type.to_string(), self.lexeme)
            }
        }
    }
}

impl Hash for RatexToken {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.line.hash(state);
        self.position.hash(state);
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
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
    Identifier,
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
    #[default]
    Break,
    EOF,
}

impl Eq for RatexTokenType {}

impl Display for RatexTokenType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn can_hash_token() {
        let token = RatexToken {
            token_type: RatexTokenType::Fun,
            lexeme: "function".to_string(),
            line: 10,
            position: 11,
        };

        let mut hashMap: HashMap<RatexToken, &str> = HashMap::new();

        hashMap.insert(token.clone(), "hello");

        assert_eq!(*hashMap.get(&token).unwrap(), "hello");
    }

    #[test]
    fn uses_correct_properties_for_hash() {
        let token1 = RatexToken {
            token_type: RatexTokenType::Fun,
            lexeme: "function".to_string(),
            line: 10,
            position: 11,
        };

        let token2 = RatexToken {
            token_type: RatexTokenType::Class,
            lexeme: "class".to_string(),
            line: 10,
            position: 11,
        };

        let mut hashMap: HashMap<RatexToken, &str> = HashMap::new();

        hashMap.insert(token1, "hello");

        assert_eq!(*hashMap.get(&token2).unwrap(), "hello");
    }
}
