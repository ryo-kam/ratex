use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    ast::{Object, RatexCallable},
    error::{RatexError, RatexErrorType},
    functions::RatexFunction,
    interpreter::RatexInterpreter,
};

#[derive(Debug, PartialEq, Clone)]
pub struct RatexClass {
    name: String,
    methods: HashMap<String, Rc<RatexFunction>>,
}

impl RatexClass {
    pub fn new(name: String, methods: HashMap<String, Rc<RatexFunction>>) -> Self {
        RatexClass { name, methods }
    }

    fn find_method(&self, name: &String) -> Option<Object> {
        if let Some(method) = self.methods.get(name) {
            let func = Rc::clone(method);
            return Some(Object::Function(func));
        }

        None
    }
}

impl RatexCallable for RatexClass {
    fn call(&self, _: &mut RatexInterpreter, _: Vec<Object>) -> Result<Object, RatexError> {
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
        if let Some(value) = self.fields.get(&name) {
            return Ok(value.clone());
        }

        if let Some(method) = self.klass.find_method(&name) {
            return Ok(method);
        }

        Err(RatexError {
            source: RatexErrorType::AccessUnknownField(name),
        })
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.fields.insert(name, value);
    }
}
