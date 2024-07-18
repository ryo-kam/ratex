use std::fmt::{Display, Formatter};

use crate::ast::ast_macro::ast_derive;
use crate::token::RatexToken;

mod ast_macro;

// Run this to see expanded macro
// cargo rustc --profile=check --bin=ratex -- -Zunpretty=expanded

#[derive(Clone, Debug)]
pub enum LiteralValue {
    Bool(bool),
    String(String),
    Number(f64),
    Nil,
}

ast_derive! {
    Expr,
    Binary(left: Box<Expr>, operator: RatexToken, right: Box<Expr>),
    Unary(operator: RatexToken, right: Box<Expr>),
    Literal(value: LiteralValue),
    Grouping(expr: Box<Expr>)
}

ast_derive! {
    Stmt,
    Expression(expr: Box<Expr>),
    Print(expr: Box<Expr>)
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
