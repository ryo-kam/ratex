use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{
    Assign, Binary, Block, Break, Call, Expr, ExprAccept, ExprVisitor, Expression, Fun, Grouping,
    If, Lambda, Literal, Logical, Object, Print, Return, Stmt, StmtAccept, StmtVisitor, Unary, Var,
    Variable, While,
};
use crate::environment::Environment;
use crate::error::{RatexError, RatexErrorType};
use crate::functions::{ClockFunction, RatexFunction};
use crate::token::{RatexToken, RatexTokenType as RXTT};

#[derive(Debug)]
pub struct RatexInterpreter {
    environment: Rc<RefCell<Environment>>,
    locals: HashMap<Expr, usize>,
    globals: Rc<RefCell<Environment>>,
}

impl RatexInterpreter {
    pub fn evaluate(&mut self, expr: Box<Expr>) -> Result<Object, RatexError> {
        expr.accept(self)
    }

    pub fn execute(&mut self, statement: Stmt) -> Result<(), RatexError> {
        statement.accept(self)
    }

    pub fn resolve(&mut self, expr: Expr, depth: usize) {
        self.locals.insert(expr, depth);
    }

    pub fn execute_block(
        &mut self,
        statements: Vec<Stmt>,
        env: Rc<RefCell<Environment>>,
    ) -> Result<(), RatexError> {
        let old_environment = Rc::clone(&self.environment);
        self.environment = env;

        for statement in statements {
            match self.execute(statement) {
                Err(e) => {
                    return Err(e);
                }
                Ok(()) => {}
            }
        }

        self.environment = old_environment;

        Ok(())
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

    pub fn new() -> Rc<RefCell<Self>> {
        let globals = Environment::new();

        globals
            .borrow_mut()
            .define("clock".to_string(), Object::Function(ClockFunction::new()));

        let environment = Rc::clone(&globals);

        Rc::new(RefCell::new(RatexInterpreter {
            environment,
            locals: HashMap::new(),
            globals,
        }))
    }

    fn look_up_variable(&self, name: RatexToken, expr: Expr) -> Result<Object, RatexError> {
        let res = self.locals.get(&expr);

        if let Some(distance) = res {
            Ok(Environment::get_at(
                Rc::clone(&self.environment),
                *distance,
                name.lexeme,
            ))
        } else {
            Ok(self.globals.borrow().get(name.lexeme)?)
        }
    }
}

impl ExprVisitor<Object> for RatexInterpreter {
    fn visit_binary(&mut self, target: &Binary) -> Result<Object, RatexError> {
        let left: Object = self.evaluate(target.left.clone())?;
        let right: Object = self.evaluate(target.right.clone())?;

        match (left, right) {
            (Object::Number(n1), Object::Number(n2)) => match target.operator.token_type {
                RXTT::Minus => Ok(Object::Number(n1 - n2)),
                RXTT::Slash => Ok(Object::Number(n1 / n2)),
                RXTT::Star => Ok(Object::Number(n1 * n2)),
                RXTT::Plus => Ok(Object::Number(n1 + n2)),
                RXTT::Greater => Ok(Object::Bool(n1 > n2)),
                RXTT::GreaterEqual => Ok(Object::Bool(n1 >= n2)),
                RXTT::Less => Ok(Object::Bool(n1 < n2)),
                RXTT::LessEqual => Ok(Object::Bool(n1 <= n2)),
                RXTT::BangEqual => Ok(Object::Bool(n1 != n2)),
                RXTT::EqualEqual => Ok(Object::Bool(n1 == n2)),
                _ => Ok(Object::Nil),
            },
            (Object::String(s1), Object::String(s2)) => match target.operator.token_type {
                RXTT::Plus => Ok(Object::String(s1 + &s2)),
                RXTT::BangEqual => Ok(Object::Bool(s1 != s2)),
                RXTT::EqualEqual => Ok(Object::Bool(s1 == s2)),
                _ => Ok(Object::Nil),
            },
            (Object::Bool(b1), Object::Bool(b2)) => match target.operator.token_type {
                RXTT::Greater => Ok(Object::Bool(b1 > b2)),
                RXTT::GreaterEqual => Ok(Object::Bool(b1 >= b2)),
                RXTT::Less => Ok(Object::Bool(b1 < b2)),
                RXTT::LessEqual => Ok(Object::Bool(b1 <= b2)),
                RXTT::BangEqual => Ok(Object::Bool(b1 != b2)),
                RXTT::EqualEqual => Ok(Object::Bool(b1 == b2)),
                _ => Ok(Object::Nil),
            },
            _ => Ok(Object::Nil),
        }
    }

    fn visit_unary(&mut self, target: &Unary) -> Result<Object, RatexError> {
        let right: Object = self.evaluate(target.right.clone())?;

        match target.operator.token_type {
            RXTT::Minus => match right {
                Object::Bool(b) => Ok(Object::Bool(!b)),
                Object::Number(n) => Ok(Object::Number(-n)),
                _ => Ok(Object::Nil),
            },
            RXTT::Bang => match right {
                Object::Bool(b) => Ok(Object::Bool(b)),
                Object::String(_) | Object::Number(_) => Ok(Object::Bool(true)),
                _ => Ok(Object::Nil),
            },
            _ => Ok(Object::Nil),
        }
    }

    fn visit_logical(&mut self, target: &Logical) -> Result<Object, RatexError> {
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

    fn visit_literal(&mut self, target: &Literal) -> Result<Object, RatexError> {
        Ok(target.value.clone())
    }

    fn visit_grouping(&mut self, target: &Grouping) -> Result<Object, RatexError> {
        self.evaluate(target.expr.clone())
    }

    fn visit_variable(&mut self, target: &Variable) -> Result<Object, RatexError> {
        return self.look_up_variable(target.name.clone(), Expr::Variable(target.clone()));
    }

    fn visit_assign(&mut self, target: &Assign) -> Result<Object, RatexError> {
        let value = self.evaluate(target.value.clone())?;
        let distance = self.locals.get(&Expr::Assign(target.clone()));

        if let Some(d) = distance {
            Environment::assign_at(
                Rc::clone(&self.environment),
                *d,
                target.name.lexeme.clone(),
                value.clone(),
            );
        } else {
            self.environment
                .borrow_mut()
                .assign(target.name.lexeme.clone(), value.clone())?;
        }
        Ok(value)
    }

    fn visit_call(&mut self, target: &Call) -> Result<Object, RatexError> {
        let callee = self.evaluate(target.callee.clone())?;

        let mut arguments = Vec::new();

        for argument in &target.arguments {
            arguments.push(self.evaluate(Box::new(argument.clone()))?);
        }

        if let Object::Function(fun) = callee {
            if arguments.len() == fun.arity()? {
                match fun.call(self, arguments) {
                    Ok(obj) => return Ok(obj),
                    Err(e) => {
                        if let RatexErrorType::Return(obj) = e.source {
                            return Ok(obj);
                        }
                        return Err(e);
                    }
                }
            } else {
                return Err(RatexError {
                    source: RatexErrorType::IncompatibleArity,
                });
            }
        }

        Err(RatexError {
            source: RatexErrorType::InvalidFunctionCall,
        })
    }

    fn visit_lambda(&mut self, target: &Lambda) -> Result<Object, RatexError> {
        let stmt = Stmt::Fun(Fun {
            name: RatexToken::default(),
            params: target.params.clone(),
            body: target.body.clone(),
        });

        let function = Object::Function(RatexFunction::new(stmt, Rc::clone(&self.environment)));

        Ok(function)
    }
}

impl StmtVisitor<()> for RatexInterpreter {
    fn visit_block(&mut self, target: &Block) -> Result<(), RatexError> {
        let block_env = Environment::new_child(Rc::clone(&self.environment));

        self.execute_block(target.statements.clone(), block_env)?;

        Ok(())
    }

    fn visit_expression(&mut self, target: &Expression) -> Result<(), RatexError> {
        self.evaluate(target.expr.clone())?;
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

    fn visit_fun(&mut self, target: &Fun) -> Result<(), RatexError> {
        let stmt = Stmt::Fun(target.clone());

        let function = Object::Function(RatexFunction::new(
            stmt,
            Environment::new_child(Rc::clone(&self.environment)),
        ));

        self.environment
            .borrow_mut()
            .define(target.name.lexeme.clone(), function);

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

    fn visit_print(&mut self, target: &Print) -> Result<(), RatexError> {
        let value = self.evaluate(target.expr.clone())?;
        println!("{value}");
        Ok(())
    }

    fn visit_return(&mut self, target: &Return) -> Result<(), RatexError> {
        Err(RatexError {
            source: RatexErrorType::Return(self.evaluate(target.value.clone())?),
        })
    }

    fn visit_var(&mut self, target: &Var) -> Result<(), RatexError> {
        let mut value = Object::Nil;

        match *target.initialiser {
            Expr::Empty => {}
            _ => {
                value = self.evaluate(target.initialiser.clone())?;
            }
        }

        match &target.name.token_type {
            RXTT::Identifier => self
                .environment
                .borrow_mut()
                .define(target.name.lexeme.clone(), value),
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
}
