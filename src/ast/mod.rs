use std::cell::RefCell;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::rc::Rc;

use crate::ast::ast_macro::ast_derive;
use crate::class::{RatexClass, RatexInstance};
use crate::interpreter::RatexInterpreter;
use crate::token::RatexToken;
use crate::RatexError;

mod ast_macro;

#[derive(Debug)]
pub enum Object {
    Bool(bool),
    String(String),
    Number(f64),
    Function(Rc<RefCell<dyn RatexCallable>>),
    Class(RatexClass),
    Instance(Rc<RefCell<RatexInstance>>),
    Nil,
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        match self {
            Object::Bool(b) => return *b,
            Object::String(s) => return s.len() > 0,
            Object::Number(n) => return *n != 0.0,
            Object::Function(_) => return true,
            Object::Class(_) => return true,
            Object::Instance(_) => return true,
            Object::Nil => return false,
        }
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

impl Eq for Object {}

impl Clone for Object {
    fn clone(&self) -> Self {
        match self {
            Object::Bool(b) => Object::Bool(b.clone()),
            Object::String(s) => Object::String(s.clone()),
            Object::Number(n) => Object::Number(n.clone()),
            Object::Function(f) => Object::Function(Rc::clone(&f)),
            Object::Class(c) => Object::Class(c.clone()),
            Object::Instance(i) => Object::Instance(i.clone()),
            Object::Nil => Object::Nil,
        }
    }
}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Object::Bool(b1), Object::Bool(b2)) => b1 == b2,
            (Object::String(s1), Object::String(s2)) => s1 == s2,
            (Object::Number(n1), Object::Number(n2)) => n1 == n2,
            (Object::Function(f1), Object::Function(f2)) => Rc::ptr_eq(f1, f2),
            (Object::Class(c1), Object::Class(c2)) => c1 == c2,
            (Object::Instance(i1), Object::Instance(i2)) => i1 == i2,
            (Object::Nil, Object::Nil) => true,
            _ => false,
        }
    }
}

ast_derive! {
    Expr,
    Binary(left: Rc<Expr>, operator: RatexToken, right: Rc<Expr>),
    Logical(left: Rc<Expr>, operator: RatexToken, right: Rc<Expr>),
    Set(object: Rc<Expr>, name: RatexToken, value: Rc<Expr>),
    This(keyword: RatexToken),
    Unary(operator: RatexToken, right: Rc<Expr>),
    Literal(value: Object),
    Grouping(expr: Rc<Expr>),
    Variable(name: RatexToken),
    Assign(name: RatexToken, value: Rc<Expr>),
    Call(callee: Rc<Expr>, paren: RatexToken, arguments: Vec<Rc<Expr>>),
    Get(object: Rc<Expr>, name: RatexToken),
    Lambda(params: Vec<RatexToken>, body: Vec<Rc<Stmt>>)
}

ast_derive! {
    Stmt,
    Block(statements: Vec<Rc<Stmt>>),
    Class(name: RatexToken, methods: Vec<Rc<Stmt>>),
    Expression(expr: Rc<Expr>),
    If(condition: Rc<Expr>, then_stmt: Rc<Stmt>, else_stmt: Rc<Stmt>),
    Fun(name: RatexToken, params: Vec<RatexToken>, body: Vec<Rc<Stmt>>),
    While(condition: Rc<Expr>, body: Rc<Stmt>),
    Break(),
    Print(expr: Rc<Expr>),
    Return(keyword: RatexToken, value: Rc<Expr>),
    Var(name: RatexToken, initialiser: Rc<Expr>)
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Object::Bool(b) => write!(f, "{b}"),
            Object::String(s) => write!(f, "{s}"),
            Object::Number(n) => write!(f, "{n}"),
            Object::Function(fun) => write!(f, "<function {}>", fun.borrow().name()),
            Object::Class(c) => write!(f, "<class {}>", c.name()),
            Object::Instance(i) => write!(f, "<{} class instance>", i.borrow().name()),
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

    fn name(&self) -> String;
}
