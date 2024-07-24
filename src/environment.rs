use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::Object,
    error::{RatexError, RatexErrorType},
};

#[derive(Clone, Debug)]
pub struct Environment {
    values: HashMap<String, Object>,
    enclosing: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: None,
        }))
    }

    pub fn new_child(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Environment>> {
        Rc::new(RefCell::new(Environment {
            values: HashMap::new(),
            enclosing: Some(parent),
        }))
    }

    pub fn define(&mut self, name: String, value: Object) {
        self.values.insert(name, value);
    }

    // pub fn get_enclosing(&self) -> Option<Box<Environment>> {
    //     self.enclosing.clone()
    // }

    pub fn get(&self, name: String) -> Result<Object, RatexError> {
        match self.values.get(&name) {
            Some(value) => Ok(value.clone()),
            None => match &self.enclosing {
                Some(parent) => {
                    let a = parent.borrow().get(name)?.clone();
                    return Ok(a);
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
                    return parent.borrow_mut().assign(name, value);
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

    pub fn get_ref(&self) -> Rc<RefCell<Environment>> {
        if let Some(e) = &self.enclosing {
            return Rc::clone(&e);
        } else {
            return Environment::new();
        }
    }
}
