use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::ast::ast_macro::ast_derive;
use crate::interpreter::RatexInterpreter;
use crate::token::RatexToken;
use crate::RatexError;

mod ast_macro;

#[derive(Debug)]
pub enum Object {
    Bool(bool),
    String(String),
    Number(f64),
    Function(Rc<dyn RatexCallable>),
    Nil,
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Bool(b) => return *b,
            Object::String(s) => return s.len() > 0,
            Object::Number(n) => return *n != 0.0,
            Object::Function(_) => return true,
            Object::Nil => return false,
        }
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Object::Bool(b) => Object::Bool(b.clone()),
            Object::String(s) => Object::String(s.clone()),
            Object::Number(n) => Object::Number(n.clone()),
            Object::Function(f) => Object::Function(Rc::clone(&f)),
            Object::Nil => Object::Nil,
        }
    }
}

ast_derive! {
    Expr,
    Binary(left: Box<Expr>, operator: RatexToken, right: Box<Expr>),
    Unary(operator: RatexToken, right: Box<Expr>),
    Logical(left: Box<Expr>, operator: RatexToken, right: Box<Expr>),
    Literal(value: Object),
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
    Return(keyword: RatexToken, value: Box<Expr>),
    Var(name: RatexToken, initialiser: Box<Expr>)
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Object::Bool(b) => write!(f, "{b}"),
            Object::String(s) => write!(f, "{s}"),
            Object::Number(n) => write!(f, "{n}"),
            Object::Function(_) => write!(f, ""),
            Object::Nil => write!(f, "Nil"),
        }
    }
}

pub trait RatexCallable: Debug {
    fn call(
        &self,
        interpreter: &mut RatexInterpreter,
        arguments: Vec<Object>,
    ) -> Result<Object, RatexError>;

    fn arity(&self) -> Result<usize, RatexError>;
}
