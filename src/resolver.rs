use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
};

use crate::{
    ast::{
        Assign, Binary, Block, Break, Call, Class, Expr, ExprAccept, ExprVisitor, Expression, Fun,
        Get, Grouping, If, Lambda, Literal, Logical, Print, Return, Set, Stmt, StmtAccept,
        StmtVisitor, Unary, Var, Variable, While,
    },
    error::{RatexError, RatexErrorType},
    interpreter::RatexInterpreter,
    token::RatexToken as RXT,
};

#[derive(Debug, Clone)]
pub enum FunctionType {
    Function,
    None,
    Method,
}

#[derive(Debug)]
pub struct Resolver {
    interpreter: Rc<RefCell<RatexInterpreter>>,
    scopes: VecDeque<RefCell<HashMap<String, bool>>>,
    current_function: FunctionType,
}

impl Resolver {
    pub fn new(interpreter: Rc<RefCell<RatexInterpreter>>) -> Self {
        Resolver {
            interpreter,
            scopes: VecDeque::new(),
            current_function: FunctionType::None,
        }
    }

    pub fn resolve_list(&mut self, statements: Vec<Stmt>) -> Result<(), RatexError> {
        for statement in statements {
            self.resolve_stmt(statement)?;
        }
        Ok(())
    }

    fn resolve_stmt(&mut self, stmt: Stmt) -> Result<(), RatexError> {
        Ok(stmt.accept(self)?)
    }

    fn resolve_expr(&mut self, expr: Expr) -> Result<(), RatexError> {
        Ok(expr.accept(self)?)
    }

    fn begin_scope(&mut self) {
        self.scopes.push_back(RefCell::new(HashMap::new()));
    }

    fn end_scope(&mut self) {
        self.scopes.pop_back();
    }

    fn declare(&self, name: RXT) -> Result<(), RatexError> {
        if self.scopes.is_empty() {
            return Ok(());
        }

        let mut map = self.scopes.back().unwrap().borrow_mut();

        if map.contains_key(&name.lexeme) {
            Err(RatexError {
                source: RatexErrorType::RedeclareLocalVariable(name.line),
            })
        } else {
            map.insert(name.lexeme, false);
            Ok(())
        }
    }

    fn define(&self, name: RXT) {
        if self.scopes.is_empty() {
            return;
        }

        self.scopes
            .back()
            .unwrap()
            .borrow_mut()
            .insert(name.lexeme, true);
    }

    fn resolve_local(&mut self, target: Expr, name: &RXT) {
        for i in (0..self.scopes.len()).rev() {
            if self
                .scopes
                .get(i)
                .unwrap()
                .borrow()
                .contains_key(&name.lexeme)
            {
                self.interpreter
                    .borrow_mut()
                    .resolve(target, self.scopes.len() - 1 - i);
                return;
            }
        }
    }

    fn resolve_function(&mut self, fun: Fun, func_type: FunctionType) -> Result<(), RatexError> {
        let enclosing_function = self.current_function.clone();
        self.current_function = func_type;

        self.begin_scope();

        for param in fun.params {
            self.declare(param.clone())?;
            self.define(param.clone());
        }

        self.resolve_list(fun.body)?;
        self.end_scope();

        self.current_function = enclosing_function;

        Ok(())
    }
}

impl ExprVisitor<()> for Resolver {
    fn visit_binary(&mut self, target: &Binary) -> Result<(), RatexError> {
        self.resolve_expr(*target.left.clone())?;
        self.resolve_expr(*target.right.clone())?;
        Ok(())
    }

    fn visit_unary(&mut self, target: &Unary) -> Result<(), RatexError> {
        self.resolve_expr(*target.right.clone())?;
        Ok(())
    }

    fn visit_logical(&mut self, target: &Logical) -> Result<(), RatexError> {
        self.resolve_expr(*target.left.clone())?;
        self.resolve_expr(*target.right.clone())?;
        Ok(())
    }

    fn visit_literal(&mut self, _: &Literal) -> Result<(), RatexError> {
        Ok(())
    }

    fn visit_grouping(&mut self, target: &Grouping) -> Result<(), RatexError> {
        self.resolve_expr(*target.expr.clone())?;
        Ok(())
    }

    fn visit_variable(&mut self, target: &Variable) -> Result<(), RatexError> {
        if self.scopes.len() > 0 {
            if let Some(b) = self
                .scopes
                .back()
                .unwrap()
                .borrow()
                .get(&target.name.lexeme)
            {
                if !b {
                    return Err(RatexError {
                        source: RatexErrorType::Break,
                    });
                }
            }

            self.resolve_local(Expr::Variable(target.clone()), &target.name);
        }

        Ok(())
    }

    fn visit_assign(&mut self, target: &Assign) -> Result<(), RatexError> {
        self.resolve_expr(*target.value.clone())?;
        self.resolve_local(Expr::Assign(target.clone()), &target.name);
        Ok(())
    }

    fn visit_call(&mut self, target: &Call) -> Result<(), RatexError> {
        self.resolve_expr(*target.callee.clone())?;

        for argument in &target.arguments {
            self.resolve_expr(argument.clone())?;
        }

        Ok(())
    }

    fn visit_lambda(&mut self, target: &Lambda) -> Result<(), RatexError> {
        for statement in &target.body {
            self.resolve_stmt(statement.clone())?;
        }
        Ok(())
    }

    fn visit_get(&mut self, target: &Get) -> Result<(), RatexError> {
        self.resolve_expr(*target.object.clone())?;
        Ok(())
    }

    fn visit_set(&mut self, target: &Set) -> Result<(), RatexError> {
        self.resolve_expr(*target.value.clone())?;
        self.resolve_expr(*target.object.clone())?;
        Ok(())
    }

    fn visit_this(&mut self, target: &crate::ast::This) -> Result<(), RatexError> {
        self.resolve_local(Expr::This(target.clone()), &target.keyword);
        Ok(())
    }
}

impl StmtVisitor<()> for Resolver {
    fn visit_block(&mut self, target: &Block) -> Result<(), RatexError> {
        self.begin_scope();
        self.resolve_list(target.statements.clone())?;
        self.end_scope();
        Ok(())
    }

    fn visit_expression(&mut self, target: &Expression) -> Result<(), RatexError> {
        self.resolve_expr(*target.expr.clone())?;
        Ok(())
    }

    fn visit_if(&mut self, target: &If) -> Result<(), RatexError> {
        self.resolve_expr(*target.condition.clone())?;
        self.resolve_stmt(*target.then_stmt.clone())?;

        if *target.else_stmt != Stmt::Empty {
            self.resolve_stmt(*target.else_stmt.clone())?;
        }

        Ok(())
    }

    fn visit_fun(&mut self, target: &Fun) -> Result<(), RatexError> {
        self.declare(target.name.clone())?;
        self.define(target.name.clone());
        self.resolve_function(target.clone(), FunctionType::Function)?;
        Ok(())
    }

    fn visit_while(&mut self, target: &While) -> Result<(), RatexError> {
        self.resolve_expr(*target.condition.clone())?;
        self.resolve_stmt(*target.body.clone())?;

        Ok(())
    }

    fn visit_break(&mut self, _: &Break) -> Result<(), RatexError> {
        Ok(())
    }

    fn visit_print(&mut self, target: &Print) -> Result<(), RatexError> {
        self.resolve_expr(*target.expr.clone())?;
        Ok(())
    }

    fn visit_return(&mut self, target: &Return) -> Result<(), RatexError> {
        if let FunctionType::None = self.current_function {
            return Err(RatexError {
                source: RatexErrorType::InvalidReturnLocation,
            });
        }

        if *target.value != Expr::Empty {
            self.resolve_expr(*target.value.clone())?;
        }

        Ok(())
    }

    fn visit_var(&mut self, target: &Var) -> Result<(), RatexError> {
        self.declare(target.name.clone())?;

        if *target.initialiser != Expr::Empty {
            self.resolve_expr(*target.initialiser.clone())?;
        }

        self.define(target.name.clone());

        Ok(())
    }

    fn visit_class(&mut self, target: &Class) -> Result<(), RatexError> {
        self.declare(target.name.clone())?;
        self.define(target.name.clone());

        self.begin_scope();
        self.scopes
            .back()
            .unwrap()
            .borrow_mut()
            .insert("this".to_string(), true);

        for method in &target.methods {
            if let Stmt::Fun(fun) = method {
                let declaration = FunctionType::Method;
                self.resolve_function(fun.clone(), declaration)?;
            }
        }

        self.end_scope();
        Ok(())
    }
}
