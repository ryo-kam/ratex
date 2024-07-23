use std::collections::HashMap;

use crate::{
    ast::LiteralValue,
    error::{RatexError, RatexErrorType},
};

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, LiteralValue>,
    enclosing: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: None,
        }
    }

    pub fn new_child(parent: Self) -> Self {
        Environment {
            values: HashMap::new(),
            enclosing: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
    }

    pub fn get_enclosing(&self) -> Option<Box<Environment>> {
        self.enclosing.clone()
    }

    pub fn get(&self, name: String) -> Result<&LiteralValue, RatexError> {
        match self.values.get(&name) {
            Some(value) => Ok(value),
            None => match &self.enclosing {
                Some(parent) => {
                    return parent.get(name);
                }
                None => Err(RatexError {
                    source: RatexErrorType::UndefinedIdentifier(name),
                }),
            },
        }
    }

    pub fn assign(&mut self, name: String, value: LiteralValue) -> Result<(), RatexError> {
        if self.values.contains_key(&name) {
            self.values.insert(name, value);
        } else {
            match &mut self.enclosing {
                Some(parent) => {
                    return parent.assign(name, value);
                }
                None => {
                    return Err(RatexError {
                        source: RatexErrorType::UndefinedIdentifier(name),
                    })
                }
            }
        }

        Ok(())
    }
}
