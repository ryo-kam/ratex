use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{Object, RatexCallable},
    error::{RatexError, RatexErrorType},
    interpreter::RatexInterpreter,
    token::RatexToken,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RatexClass {
    name: String,
}

impl RatexClass {
    pub fn new(name: String) -> Self {
        RatexClass { name }
    }
}

impl RatexCallable for RatexClass {
    fn call(
        &self,
        interpreter: &mut RatexInterpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RatexError> {
        Ok(Object::Instance(RatexInstance::new(self.clone())))
    }

    fn arity(&self) -> Result<usize, RatexError> {
        Ok(0)
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct RatexInstance {
    klass: RatexClass,
    fields: HashMap<String, Object>,
}

impl RatexInstance {
    pub fn new(klass: RatexClass) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(RatexInstance {
            klass,
            fields: HashMap::new(),
        }))
    }

    pub fn name(&self) -> String {
        self.klass.name()
    }

    pub fn get(&self, name: String) -> Result<Object, RatexError> {
        match self.fields.get(&name) {
            Some(value) => return Ok(value.clone()),
            None => {
                return Err(RatexError {
                    source: RatexErrorType::AccessUnknownField(name),
                })
            }
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.fields.insert(name, value);
    }
}
