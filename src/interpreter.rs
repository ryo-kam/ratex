use crate::ast::{
    Binary, Expr, ExprAccept, ExprVisitor, Expression, Grouping, Literal, LiteralValue, Print,
    Stmt, StmtAccept, StmtVisitor, Unary, Var, Variable,
};
use crate::environment::Environment;
use crate::token::RatexTokenType as RXTT;

pub struct RatexInterpreter {
    environment: Environment,
}

impl RatexInterpreter {
    pub fn new() -> Self {
        RatexInterpreter {
            environment: Environment::new(),
        }
    }

    pub fn evaluate(&mut self, expr: Expr) -> LiteralValue {
        expr.accept(self)
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) {
        for statement in statements {
            self.execute(statement);
        }
    }

    pub fn execute(&mut self, statement: Stmt) {
        statement.accept(self);
    }
}

impl ExprVisitor<LiteralValue> for RatexInterpreter {
    fn visit_binary(&mut self, target: &Binary) -> LiteralValue {
        let left: LiteralValue = self.evaluate(*target.left.clone());
        let right: LiteralValue = self.evaluate(*target.right.clone());

        match (left, right) {
            (LiteralValue::Number(n1), LiteralValue::Number(n2)) => match target.operator.token {
                RXTT::Minus => LiteralValue::Number(n1 - n2),
                RXTT::Slash => LiteralValue::Number(n1 / n2),
                RXTT::Star => LiteralValue::Number(n1 * n2),
                RXTT::Plus => LiteralValue::Number(n1 + n2),
                RXTT::Greater => LiteralValue::Bool(n1 > n2),
                RXTT::GreaterEqual => LiteralValue::Bool(n1 >= n2),
                RXTT::Less => LiteralValue::Bool(n1 < n2),
                RXTT::LessEqual => LiteralValue::Bool(n1 <= n2),
                RXTT::BangEqual => LiteralValue::Bool(n1 != n2),
                RXTT::EqualEqual => LiteralValue::Bool(n1 == n2),
                _ => LiteralValue::Nil,
            },
            (LiteralValue::String(s1), LiteralValue::String(s2)) => match target.operator.token {
                RXTT::Plus => LiteralValue::String(s1 + &s2),
                RXTT::BangEqual => LiteralValue::Bool(s1 != s2),
                RXTT::EqualEqual => LiteralValue::Bool(s1 == s2),
                _ => LiteralValue::Nil,
            },
            (LiteralValue::Bool(b1), LiteralValue::Bool(b2)) => match target.operator.token {
                RXTT::Greater => LiteralValue::Bool(b1 > b2),
                RXTT::GreaterEqual => LiteralValue::Bool(b1 >= b2),
                RXTT::Less => LiteralValue::Bool(b1 < b2),
                RXTT::LessEqual => LiteralValue::Bool(b1 <= b2),
                RXTT::BangEqual => LiteralValue::Bool(b1 != b2),
                RXTT::EqualEqual => LiteralValue::Bool(b1 == b2),
                _ => LiteralValue::Nil,
            },
            _ => LiteralValue::Nil,
        }
    }

    fn visit_unary(&mut self, target: &Unary) -> LiteralValue {
        let right: LiteralValue = self.evaluate(*target.right.clone());

        match target.operator.token {
            RXTT::Minus => match right {
                LiteralValue::Bool(b) => LiteralValue::Bool(!b),
                LiteralValue::Number(n) => LiteralValue::Number(-n),
                _ => LiteralValue::Nil,
            },
            RXTT::Bang => match right {
                LiteralValue::Bool(b) => LiteralValue::Bool(b),
                LiteralValue::String(_) | LiteralValue::Number(_) => LiteralValue::Bool(true),
                _ => LiteralValue::Nil,
            },
            _ => LiteralValue::Nil,
        }
    }

    fn visit_variable(&mut self, target: &Variable) -> LiteralValue {
        match &target.name.token {
            RXTT::Identifier(s) => return self.environment.get(s.to_string()).unwrap().clone(),
            _ => panic!("Expected Identifier"),
        }
    }

    fn visit_literal(&mut self, target: &Literal) -> LiteralValue {
        target.value.clone()
    }

    fn visit_grouping(&mut self, target: &Grouping) -> LiteralValue {
        self.evaluate(*target.expr.clone())
    }
}

impl StmtVisitor<()> for RatexInterpreter {
    fn visit_expression(&mut self, target: &Expression) {
        self.evaluate(*target.expr.clone());
    }

    fn visit_print(&mut self, target: &Print) {
        let value = self.evaluate(*target.expr.clone());

        println!("{value}");
    }

    fn visit_var(&mut self, target: &Var) {
        let mut value = LiteralValue::Nil;

        match *target.initialiser {
            Expr::Empty => {}
            _ => {
                value = self.evaluate(*target.initialiser.clone());
            }
        }

        match &target.name.token {
            RXTT::Identifier(s) => self.environment.define(s.to_string(), value),
            _ => panic!("Expected Identifier"),
        }
    }
}
