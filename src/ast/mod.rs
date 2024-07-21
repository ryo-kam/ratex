use std::fmt::{Display, Formatter};

use crate::ast::ast_macro::ast_derive;
use crate::token::RatexToken;
use crate::RatexError;

mod ast_macro;

// Run this to see expanded macro
// cargo rustc --profile=check --bin=ratex -- -Zunpretty=expanded

#[derive(Clone, Debug, PartialEq)]
pub enum LiteralValue {
    Bool(bool),
    String(String),
    Number(f64),
    Nil,
}

impl LiteralValue {
    pub fn is_truthy(&self) -> bool {
        match self {
            LiteralValue::Bool(b) => return *b,
            LiteralValue::String(s) => return s.len() > 0,
            LiteralValue::Number(n) => return *n != 0.0,
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
    Assign(name: RatexToken, value: Box<Expr>)
}

ast_derive! {
    Stmt,
    Block(statements: Vec<Stmt>),
    Expression(expr: Box<Expr>),
    If(condition: Box<Expr>, then_stmt: Box<Stmt>, else_stmt: Box<Stmt>),
    While(condition: Box<Expr>, body: Box<Stmt>),
    Print(expr: Box<Expr>),
    Var(name: RatexToken, initialiser: Box<Expr>)
}

impl Display for LiteralValue {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            LiteralValue::Bool(b) => write!(f, "{b}"),
            LiteralValue::String(s) => write!(f, "{s}"),
            LiteralValue::Number(n) => write!(f, "{n}"),
            LiteralValue::Nil => write!(f, "Nil"),
        }
    }
}
