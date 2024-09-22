use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{Object, RatexCallable, Stmt},
    class::RatexInstance,
    environment::Environment,
    error::{RatexError, RatexErrorType},
    interpreter::RatexInterpreter,
};

#[derive(PartialEq, Clone)]
pub struct RatexFunction {
    name: String,
    declaration: Rc<Stmt>,
    closure: Rc<RefCell<Environment>>,
}

impl Debug for RatexFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Stmt::Fun(fun) = &*self.declaration {
            let mut closure_placeholder = self.closure.borrow().clone();

            closure_placeholder
                .assign(fun.name.lexeme.clone(), Object::Nil)
                .unwrap();

            f.debug_struct("RatexFunction")
                .field("declaration", &*self.declaration)
                .field("closure", &closure_placeholder)
                .finish()
        } else {
            panic!("Function not found");
        }
    }
}

impl RatexCallable for RatexFunction {
    fn call(
        &self,
        interpreter: &mut RatexInterpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RatexError> {
        match &*self.declaration {
            Stmt::Fun(f) => {
                for i in 0..f.params.len() {
                    self.closure.borrow_mut().define(
                        f.params.get(i).unwrap().lexeme.clone(),
                        arguments.get(i).unwrap().clone(),
                    );
                }

                interpreter.execute_block(f.body.clone(), Rc::clone(&self.closure))?;
                Ok(Object::Nil)
            }
            _ => Err(RatexError {
                source: RatexErrorType::InvalidFunctionCall,
            }),
        }
    }

    fn arity(&self) -> Result<usize, RatexError> {
        match &*self.declaration {
            Stmt::Fun(f) => Ok(f.params.len()),
            _ => Err(RatexError {
                source: RatexErrorType::InvalidFunctionCall,
            }),
        }
    }

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl RatexFunction {
    pub fn new(
        name: String,
        declaration: Rc<Stmt>,
        closure: Rc<RefCell<Environment>>,
    ) -> Rc<RefCell<RatexFunction>> {
        Rc::new(RefCell::new(RatexFunction {
            name,
            closure,
            declaration,
        }))
    }

    pub fn bind(&mut self, instance: RatexInstance) {
        let env = Environment::new_child(Rc::clone(&self.closure));
        env.borrow_mut().define(
            "this".to_owned(),
            Object::Instance(Rc::new(RefCell::new(instance))),
        );

        self.closure = env;
    }
}

#[derive(Debug)]
pub struct ClockFunction {}

impl RatexCallable for ClockFunction {
    fn call(&self, _: &mut RatexInterpreter, _: Vec<Object>) -> Result<Object, RatexError> {
        Ok(Object::Number(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        ))
    }

    fn arity(&self) -> Result<usize, RatexError> {
        Ok(0)
    }

    fn name(&self) -> String {
        "clock".to_string()
    }
}

impl ClockFunction {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(ClockFunction {}))
    }
}
