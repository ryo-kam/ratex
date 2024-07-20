use std::{collections::HashMap, rc::Rc};

use crate::{
    ast::LiteralValue,
    error::{RatexError, RatexErrorType},
};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, LiteralValue>,
    enclosing: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Rc<Self> {
        Rc::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        })
    }

    pub fn new_child(parent: Rc<Self>) -> Rc<Self> {
        Rc::new(Environment {
            values: HashMap::new(),
            enclosing: Some(parent),
        })
    }

    pub fn define(&mut self, name: String, value: LiteralValue) {
        self.values.insert(name, value);
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
            return Ok(());
        }

        Err(RatexError {
            source: RatexErrorType::UndefinedIdentifier(name),
        })
    }
}
