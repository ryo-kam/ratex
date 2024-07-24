use std::{
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    ast::{Object, RatexCallable, Stmt},
    environment::Environment,
    error::{RatexError, RatexErrorType},
    interpreter::RatexInterpreter,
};

#[derive(Debug)]
pub struct RatexFunction {
    declaration: Box<Stmt>,
}

impl RatexCallable for RatexFunction {
    fn call(
        &self,
        interpreter: &mut RatexInterpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RatexError> {
        let mut environment = Environment::new();

        match &*self.declaration {
            Stmt::Fun(f) => {
                for i in 0..f.params.len() {
                    environment.define(
                        f.params.get(i).unwrap().lexeme.clone(),
                        arguments.get(i).unwrap().clone(),
                    );
                }

                interpreter.execute_block(f.body.clone(), environment)?;
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
    pub fn new(stmt: Stmt) -> Rc<Self> {
        Rc::new(RatexFunction {
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
