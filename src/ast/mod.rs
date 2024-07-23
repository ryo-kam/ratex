use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::ast::ast_macro::ast_derive;
use crate::environment::Environment;
use crate::error::RatexErrorType;
use crate::interpreter::RatexInterpreter;
use crate::token::RatexToken;
use crate::{environment, RatexError};

mod ast_macro;

// Run this to see expanded macro
// cargo rustc --profile=check --bin=ratex -- -Zunpretty=expanded

#[derive(Clone)]
pub enum LiteralValue {
    Bool(bool),
    String(String),
    Number(f64),
    Function(RatexFunction),
    Nil,
}

impl LiteralValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Bool(b) => return *b,
            LiteralValue::String(s) => return s.len() > 0,
            LiteralValue::Number(n) => return *n != 0.0,
            LiteralValue::Function(_) => return true,
            LiteralValue::Nil => return false,
        }
    }
}

ast_derive! {
    Expr,
    Binary(left: Box<Expr>, operator: RatexToken, right: Box<Expr>),
    Unary(operator: RatexToken, right: Box<Expr>),
    Logical(left: Box<Expr>, operator: RatexToken, right: Box<Expr>),
    Literal(value: LiteralValue),
    Grouping(expr: Box<Expr>),
    Variable(name: RatexToken),
    Assign(name: RatexToken, value: Box<Expr>),
    Call(callee: Box<Expr>, paren: RatexToken, arguments: Vec<Expr>)
}

ast_derive! {
    Stmt,
    Block(statements: Vec<Stmt>),
    Expression(expr: Box<Expr>),
    If(condition: Box<Expr>, then_stmt: Box<Stmt>, else_stmt: Box<Stmt>),
    Fun(name: RatexToken, params: Vec<RatexToken>, body: Vec<Stmt>),
    While(condition: Box<Expr>, body: Box<Stmt>),
    Break(),
    Print(expr: Box<Expr>),
    Var(name: RatexToken, initialiser: Box<Expr>)
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LiteralValue::Bool(b) => write!(f, "{b}"),
            LiteralValue::String(s) => write!(f, "{s}"),
            LiteralValue::Number(n) => write!(f, "{n}"),
            LiteralValue::Function(rf) => {
                if let Stmt::Fun(fun) = &*rf.declaration {
                    write!(f, "<fn {}>", fun.name.lexeme)
                } else {
                    write!(f, "Invalid function")
                }
            }
            LiteralValue::Nil => write!(f, "Nil"),
        }
    }
}

#[derive(Clone)]
pub struct RatexFunction {
    declaration: Box<Stmt>,
}

impl RatexFunction {
    pub fn call(
        &self,
        interpreter: &mut RatexInterpreter,
        arguments: Vec<LiteralValue>,
    ) -> Result<(), RatexError> {
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
                Ok(())
            }
            _ => Err(RatexError {
                source: RatexErrorType::InvalidFunctionCall,
            }),
        }
    }

    pub fn new(stmt: Stmt) -> Self {
        RatexFunction {
            declaration: Box::new(stmt),
        }
    }

    pub fn arity(&self) -> Result<usize, RatexError> {
        match &*self.declaration {
            Stmt::Fun(f) => Ok(f.params.len()),
            _ => Err(RatexError {
                source: RatexErrorType::InvalidFunctionCall,
            }),
        }
    }
}
