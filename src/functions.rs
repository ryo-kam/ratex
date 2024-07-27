use std::{
    cell::RefCell,
    fmt::Debug,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{Object, RatexCallable, Stmt},
    environment::Environment,
    error::{RatexError, RatexErrorType},
    interpreter::RatexInterpreter,
};

pub struct RatexFunction {
    declaration: Box<Stmt>,
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
}

impl RatexFunction {
    pub fn new(stmt: Stmt, closure: Rc<RefCell<Environment>>) -> Rc<Self> {
        Rc::new(RatexFunction {
            closure,
            declaration: Box::new(stmt),
        })
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
}

impl ClockFunction {
    pub fn new() -> Rc<Self> {
        Rc::new(ClockFunction {})
    }
}
