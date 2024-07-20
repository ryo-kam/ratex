use std::collections::HashMap;

use crate::{
    ast::LiteralValue,
    error::{RatexError, RatexErrorType},
};

pub struct Environment {
    values: HashMap<String, LiteralValue>,
}

impl Environment {
    pub fn new() -> Self {
        return Environment {
            values: HashMap::new(),
        };
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: String) -> Result<&LiteralValue, RatexError> {
        match self.values.get(&name) {
            Some(value) => Ok(value),
            None => Err(RatexError {
                source: RatexErrorType::UndefinedIdentifier(name),
            }),
        }
    }
}
