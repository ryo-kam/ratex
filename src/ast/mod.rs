use crate::ast::ast_macro::ast_derive;
use crate::token::RatexToken;

mod ast_macro;

// Run this to see expanded macro
// cargo rustc --profile=check --bin=ratex -- -Zunpretty=expanded

ast_derive! {
    Binary(left: Box<Expr>, operator: RatexToken, right: Box<Expr>),
    Unary(operator: RatexToken, right: Box<Expr>),
    Literal(value: LiteralValue),
    Grouping(expr: Box<Expr>)
}

pub struct AstPrinter {}

impl AstPrinter {
    pub fn print(&mut self, expr: Expr) -> String {
        return expr.accept(self);
    }

    fn parenthesize_three(&mut self, operator: String, left: &Expr, right: &Expr) -> String {
        format!(
            "({} {} {})",
            operator,
            left.accept(self),
            right.accept(self)
        )
    }

    fn parenthesize_two(&mut self, operator: String, left: &Expr) -> String {
        format!("({} {})", operator, left.accept(self),)
    }
}

impl AstVisitor<String> for AstPrinter {
    fn visit_binary(&mut self, expr: &Binary) -> String {
        self.parenthesize_three(expr.operator.lexeme.clone(), &expr.left, &expr.right)
    }
    fn visit_literal(&mut self, expr: &Literal) -> String {
        match &expr.value {
            LiteralValue::String(s) => s.clone(),
            LiteralValue::Number(n) => n.to_string(),
            LiteralValue::Nil => "nil".to_owned(),
        }
    }
    fn visit_grouping(&mut self, expr: &Grouping) -> String {
        self.parenthesize_two("group".to_owned(), &expr.expr)
    }
    fn visit_unary(&mut self, expr: &Unary) -> String {
        self.parenthesize_two(expr.operator.lexeme.clone(), &expr.right)
    }
}
