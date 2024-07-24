use std::collections::HashMap;

use crate::{
    ast::Object,
    error::{RatexError, RatexErrorType},
};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, Object>,
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

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    pub fn get_enclosing(&self) -> Option<Box<Environment>> {
        self.enclosing.clone()
    }

    pub fn get(&self, name: String) -> Result<&Object, RatexError> {
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

    pub fn assign(&mut self, name: String, value: Object) -> Result<(), RatexError> {
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
