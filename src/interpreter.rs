use crate::ast::{
    Assign, Binary, Block, Break, Call, Expr, ExprAccept, ExprVisitor, Expression, Fun, Grouping,
    If, Literal, LiteralValue, Logical, Print, RatexFunction, Stmt, StmtAccept, StmtVisitor, Unary,
    Var, Variable, While,
};
use crate::environment::Environment;
use crate::error::{RatexError, RatexErrorType};
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

    pub fn evaluate(&mut self, expr: Box<Expr>) -> Result<LiteralValue, RatexError> {
        expr.accept(self)
    }

    pub fn interpret(&mut self, statements: Vec<Stmt>) -> Result<(), RatexError> {
        for statement in statements {
            match self.execute(statement) {
                Err(e) => match e.source {
                    RatexErrorType::Break => {}
                    _ => {
                        return Err(e);
                    }
                },
                _ => {}
            };
        }

        Ok(())
    }

    pub fn execute(&mut self, statement: Stmt) -> Result<(), RatexError> {
        statement.accept(self)
    }

    pub fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        env: Environment,
    ) -> Result<(), RatexError> {
        let previous = self.environment.clone();

        self.environment = env;

        for statement in statements {
            match self.execute(statement) {
                Err(e) => {
                    return Err(e);
                }
                Ok(()) => {}
            }
        }

        if let Some(parent) = self.environment.get_enclosing() {
            self.environment = *parent;
        }

        Ok(())
    }
}

impl ExprVisitor<LiteralValue> for RatexInterpreter {
    fn visit_binary(&mut self, target: &Binary) -> Result<LiteralValue, RatexError> {
        let left: LiteralValue = self.evaluate(target.left.clone())?;
        let right: LiteralValue = self.evaluate(target.right.clone())?;

        match (left, right) {
            (LiteralValue::Number(n1), LiteralValue::Number(n2)) => {
                match target.operator.token_type {
                    RXTT::Minus => Ok(LiteralValue::Number(n1 - n2)),
                    RXTT::Slash => Ok(LiteralValue::Number(n1 / n2)),
                    RXTT::Star => Ok(LiteralValue::Number(n1 * n2)),
                    RXTT::Plus => Ok(LiteralValue::Number(n1 + n2)),
                    RXTT::Greater => Ok(LiteralValue::Bool(n1 > n2)),
                    RXTT::GreaterEqual => Ok(LiteralValue::Bool(n1 >= n2)),
                    RXTT::Less => Ok(LiteralValue::Bool(n1 < n2)),
                    RXTT::LessEqual => Ok(LiteralValue::Bool(n1 <= n2)),
                    RXTT::BangEqual => Ok(LiteralValue::Bool(n1 != n2)),
                    RXTT::EqualEqual => Ok(LiteralValue::Bool(n1 == n2)),
                    _ => Ok(LiteralValue::Nil),
                }
            }
            (LiteralValue::String(s1), LiteralValue::String(s2)) => {
                match target.operator.token_type {
                    RXTT::Plus => Ok(LiteralValue::String(s1 + &s2)),
                    RXTT::BangEqual => Ok(LiteralValue::Bool(s1 != s2)),
                    RXTT::EqualEqual => Ok(LiteralValue::Bool(s1 == s2)),
                    _ => Ok(LiteralValue::Nil),
                }
            }
            (LiteralValue::Bool(b1), LiteralValue::Bool(b2)) => match target.operator.token_type {
                RXTT::Greater => Ok(LiteralValue::Bool(b1 > b2)),
                RXTT::GreaterEqual => Ok(LiteralValue::Bool(b1 >= b2)),
                RXTT::Less => Ok(LiteralValue::Bool(b1 < b2)),
                RXTT::LessEqual => Ok(LiteralValue::Bool(b1 <= b2)),
                RXTT::BangEqual => Ok(LiteralValue::Bool(b1 != b2)),
                RXTT::EqualEqual => Ok(LiteralValue::Bool(b1 == b2)),
                _ => Ok(LiteralValue::Nil),
            },
            _ => Ok(LiteralValue::Nil),
        }
    }

    fn visit_unary(&mut self, target: &Unary) -> Result<LiteralValue, RatexError> {
        let right: LiteralValue = self.evaluate(target.right.clone())?;

        match target.operator.token_type {
            RXTT::Minus => match right {
                LiteralValue::Bool(b) => Ok(LiteralValue::Bool(!b)),
                LiteralValue::Number(n) => Ok(LiteralValue::Number(-n)),
                _ => Ok(LiteralValue::Nil),
            },
            RXTT::Bang => match right {
                LiteralValue::Bool(b) => Ok(LiteralValue::Bool(b)),
                LiteralValue::String(_) | LiteralValue::Number(_) => Ok(LiteralValue::Bool(true)),
                _ => Ok(LiteralValue::Nil),
            },
            _ => Ok(LiteralValue::Nil),
        }
    }

    fn visit_variable(&mut self, target: &Variable) -> Result<LiteralValue, RatexError> {
        match &target.name.token_type {
            RXTT::Identifier => Ok(self.environment.get(target.name.lexeme.clone())?.clone()),
            _ => Err(RatexError {
                source: RatexErrorType::ExpectedToken(target.name.line, "Identifier".to_owned()),
            }),
        }
    }

    fn visit_literal(&mut self, target: &Literal) -> Result<LiteralValue, RatexError> {
        Ok(target.value.clone())
    }

    fn visit_grouping(&mut self, target: &Grouping) -> Result<LiteralValue, RatexError> {
        self.evaluate(target.expr.clone())
    }

    fn visit_assign(&mut self, target: &Assign) -> Result<LiteralValue, RatexError> {
        let value = self.evaluate(target.value.clone())?;
        self.environment
            .assign(target.name.lexeme.clone(), value.clone())?;
        Ok(value)
    }

    fn visit_logical(&mut self, target: &Logical) -> Result<LiteralValue, RatexError> {
        let left = self.evaluate(target.left.clone())?;

        match target.operator.token_type {
            RXTT::Or => {
                if left.is_truthy() {
                    return Ok(left);
                } else {
                    return Ok(self.evaluate(target.right.clone())?);
                }
            }
            RXTT::And => {
                if !left.is_truthy() {
                    return Ok(left);
                } else {
                    return Ok(self.evaluate(target.right.clone())?);
                }
            }
            _ => Err(RatexError {
                source: RatexErrorType::InvalidLogicalOperation(target.operator.line),
            }),
        }
    }

    fn visit_call(&mut self, target: &Call) -> Result<LiteralValue, RatexError> {
        let callee = self.evaluate(target.callee.clone())?;

        let mut arguments = Vec::new();

        for argument in &target.arguments {
            arguments.push(self.evaluate(Box::new(argument.clone()))?);
        }

        if let LiteralValue::Function(fun) = callee {
            fun.call(self, arguments)?;
            Ok(LiteralValue::Nil)
        } else {
            Err(RatexError {
                source: RatexErrorType::InvalidFunctionCall,
            })
        }
    }
}

impl StmtVisitor<()> for RatexInterpreter {
    fn visit_expression(&mut self, target: &Expression) -> Result<(), RatexError> {
        self.evaluate(target.expr.clone())?;
        Ok(())
    }

    fn visit_print(&mut self, target: &Print) -> Result<(), RatexError> {
        let value = self.evaluate(target.expr.clone())?;
        println!("{value}");
        Ok(())
    }

    fn visit_var(&mut self, target: &Var) -> Result<(), RatexError> {
        let mut value = LiteralValue::Nil;

        match *target.initialiser {
            Expr::Empty => {}
            _ => {
                value = self.evaluate(target.initialiser.clone())?;
            }
        }

        match &target.name.token_type {
            RXTT::Identifier => self.environment.define(target.name.lexeme.clone(), value),
            _ => {
                return Err(RatexError {
                    source: RatexErrorType::ExpectedToken(
                        target.name.line,
                        "Identifier".to_owned(),
                    ),
                });
            }
        }

        Ok(())
    }

    fn visit_block(&mut self, target: &Block) -> Result<(), RatexError> {
        let block_env = Environment::new_child(self.environment.clone());
        self.execute_block(target.statements.clone(), block_env)?;

        Ok(())
    }

    fn visit_if(&mut self, target: &If) -> Result<(), RatexError> {
        if self.evaluate(target.condition.clone())?.is_truthy() {
            self.execute(*target.then_stmt.clone())?
        } else {
            match *target.else_stmt {
                Stmt::Empty => {}
                _ => self.execute(*target.else_stmt.clone())?,
            }
        }
        Ok(())
    }

    fn visit_while(&mut self, target: &While) -> Result<(), RatexError> {
        while self.evaluate(target.condition.clone())?.is_truthy() {
            self.execute(*target.body.clone())?
        }

        Ok(())
    }

    fn visit_break(&mut self, _: &Break) -> Result<(), RatexError> {
        Err(RatexError {
            source: RatexErrorType::Break,
        })
    }

    fn visit_fun(&mut self, target: &Fun) -> Result<(), RatexError> {
        let stmt = Stmt::Fun(target.clone());

        let function = LiteralValue::Function(RatexFunction::new(stmt));
        self.environment
            .define(target.name.lexeme.clone(), function);

        Ok(())
    }
}
