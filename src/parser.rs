use std::{borrow::Borrow, rc::Rc};

use crate::{
    ast::{
        Assign, Binary, Block, Break, Call, Class, Expr, Expression, Fun, Get, Grouping, If,
        Lambda, Literal, Logical, Object, Print, Return, Set, Stmt, This, Unary, Var, Variable,
        While,
    },
    error::{RatexError, RatexErrorType},
    token::{RatexToken as RXT, RatexTokenType as RXTT},
};

pub struct Parser {
    tokens: Vec<RXT>,
    current: usize,
    has_error: bool,
}

impl Parser {
    pub fn new(input: Vec<RXT>) -> Self {
        Parser {
            tokens: input,
            current: 0,
            has_error: false,
        }
    }

    pub fn has_error(&self) -> bool {
        self.has_error
    }

    pub fn parse(&mut self) -> Vec<Rc<Stmt>> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.declaration() {
                Ok(stmt) => {
                    statements.push(stmt);
                }
                Err(e) => {
                    println!("Error: {}", e);
                    self.has_error = true;
                    self.synchronise();
                }
            }
        }

        statements
    }

    fn expression(&mut self) -> Result<Rc<Expr>, RatexError> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![RXTT::BangEqual, RXTT::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Binary::new(Rc::clone(&expr), operator, Rc::clone(&right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.term()?;

        while self.match_token(vec![
            RXTT::Greater,
            RXTT::GreaterEqual,
            RXTT::Less,
            RXTT::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Binary::new(Rc::clone(&expr), operator, Rc::clone(&right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.factor()?;

        while self.match_token(vec![RXTT::Minus, RXTT::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Binary::new(Rc::clone(&expr), operator, Rc::clone(&right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.unary()?;

        while self.match_token(vec![RXTT::Slash, RXTT::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Binary::new(Rc::clone(&expr), operator, Rc::clone(&right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Rc<Expr>, RatexError> {
        if self.match_token(vec![RXTT::Bang, RXTT::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Unary::new(operator, Rc::clone(&right)))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![RXTT::LeftParen]) {
                expr = self.finish_call(&expr)?;
            } else if self.match_token(vec![RXTT::Dot]) {
                let name = self.consume(RXTT::Identifier)?;
                expr = Get::new(Rc::clone(&expr), name.clone())
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Rc<Expr>, RatexError> {
        match &self.tokens.get(self.current).unwrap().token_type {
            RXTT::False => {
                self.current += 1;
                Ok(Literal::new(Object::Bool(false)))
            }
            RXTT::True => {
                self.current += 1;
                Ok(Literal::new(Object::Bool(true)))
            }
            RXTT::Nil => {
                self.current += 1;
                Ok(Literal::new(Object::Nil))
            }
            RXTT::Number(n) => {
                self.current += 1;
                Ok(Literal::new(Object::Number(n.clone())))
            }
            RXTT::String(s) => {
                self.current += 1;
                Ok(Literal::new(Object::String(s.clone())))
            }
            RXTT::LeftParen => {
                self.current += 1;
                let expr = self.expression()?;

                self.consume(RXTT::RightParen)?;

                Ok(Grouping::new(Rc::clone(&expr)))
            }
            RXTT::This => {
                self.current += 1;
                Ok(This::new(self.previous().clone()))
            }
            RXTT::Identifier => {
                self.current += 1;
                Ok(Variable::new(self.previous().clone()))
            }
            RXTT::Fun => {
                self.current += 1;
                self.anonymous_function()
            }
            _ => Err(RatexError {
                source: RatexErrorType::UnexpectedToken(
                    self.peek().line,
                    format!("{}", self.peek().lexeme),
                ),
            }),
        }
    }

    fn match_token(&mut self, vec: Vec<RXTT>) -> bool {
        for token_type in vec {
            if self.check(&token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn previous(&self) -> &RXT {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn check(&self, token_type: &RXTT) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == *token_type
    }

    fn advance(&mut self) -> &RXT {
        if !self.is_at_end() {
            self.current += 1;
        };
        self.previous()
    }

    fn peek(&self) -> &RXT {
        self.tokens.get(self.current).unwrap()
    }

    fn consume(&mut self, token_type: RXTT) -> Result<&RXT, RatexError> {
        if self.check(&token_type) {
            return Ok(self.advance());
        }

        Err(RatexError {
            source: RatexErrorType::ExpectedToken(self.previous().line, ";".to_owned()),
        })
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == RXTT::EOF
    }

    fn statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        if self.match_token(vec![RXTT::Class]) {
            return self.class_declaration();
        }

        if self.match_token(vec![RXTT::Return]) {
            return self.return_statement();
        }

        if self.match_token(vec![RXTT::Fun]) {
            return self.function_statement();
        }

        if self.match_token(vec![RXTT::For]) {
            return self.for_statement();
        }

        if self.match_token(vec![RXTT::While]) {
            return self.while_statement();
        }

        if self.match_token(vec![RXTT::If]) {
            return self.if_statement();
        }

        if self.match_token(vec![RXTT::Print]) {
            return self.print_statement();
        }

        if self.match_token(vec![RXTT::LeftBrace]) {
            return Ok(Block::new(self.block()?));
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        let value = self.expression()?;

        self.consume(RXTT::Semicolon)?;

        Ok(Print::new(value))
    }

    fn expression_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        let value = self.expression()?;

        self.consume(RXTT::Semicolon)?;

        Ok(Expression::new(value))
    }

    fn break_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        if self.match_token(vec![RXTT::Break]) {
            self.consume(RXTT::Semicolon)?;
            return Ok(Break::new());
        }

        self.declaration()
    }

    fn declaration(&mut self) -> Result<Rc<Stmt>, RatexError> {
        if self.match_token(vec![RXTT::Var]) {
            Ok(self.var_declaration()?)
        } else {
            Ok(self.statement()?)
        }
    }

    fn var_declaration(&mut self) -> Result<Rc<Stmt>, RatexError> {
        let token = &self.peek();
        let name = match token.token_type {
            RXTT::Identifier => RXT {
                token_type: RXTT::Identifier,
                lexeme: token.lexeme.clone(),
                line: 0,
            },
            _ => {
                panic!("Expected variable name.")
            }
        };

        self.advance();

        let mut initialiser = Rc::new(Expr::Empty);

        if self.match_token(vec![RXTT::Equal]) {
            initialiser = self.expression()?;
        }

        self.consume(RXTT::Semicolon)?;

        Ok(Var::new(name, Rc::clone(&initialiser)))
    }

    fn synchronise(&mut self) {
        self.advance();

        while !self.is_at_end() {
            match self.previous().token_type {
                RXTT::Semicolon => return (),
                _ => {}
            }

            match self.peek().token_type {
                RXTT::Class
                | RXTT::Fun
                | RXTT::Var
                | RXTT::For
                | RXTT::If
                | RXTT::While
                | RXTT::Print
                | RXTT::Return => return (),
                _ => {}
            }

            self.advance();
        }
    }

    fn assignment(&mut self) -> Result<Rc<Expr>, RatexError> {
        let expr = self.or()?;

        if self.match_token(vec![RXTT::Equal]) {
            let equals = self.previous();

            match expr.borrow() {
                Expr::Variable(var) => {
                    let name = &var.name;
                    let value = self.assignment()?;
                    return Ok(Assign::new(name.clone(), Rc::clone(&value)));
                }
                Expr::Get(get) => {
                    return Ok(Set::new(
                        Rc::clone(&get.object),
                        get.name.clone(),
                        Rc::clone(&self.assignment()?),
                    ))
                }
                _ => {
                    return Err(RatexError {
                        source: RatexErrorType::InvalidAssignment(equals.line),
                    });
                }
            }
        }

        Ok(expr)
    }

    fn block(&mut self) -> Result<Vec<Rc<Stmt>>, RatexError> {
        let mut statements = Vec::new();

        while !self.check(&RXTT::RightBrace) && !self.is_at_end() {
            statements.push(self.break_statement()?);
        }

        self.consume(RXTT::RightBrace)?;

        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let condition = self.expression()?;
        self.consume(RXTT::RightParen)?;

        let then_stmt = self.statement()?;

        let mut else_stmt = Rc::new(Stmt::Empty);

        if self.match_token(vec![RXTT::Else]) {
            else_stmt = self.statement()?;
        }

        Ok(If::new(
            Rc::clone(&condition),
            Rc::clone(&then_stmt),
            Rc::clone(&else_stmt),
        ))
    }

    fn or(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.and()?;

        while self.match_token(vec![RXTT::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;

            expr = Logical::new(Rc::clone(&expr), operator, Rc::clone(&right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Rc<Expr>, RatexError> {
        let mut expr = self.equality()?;

        while self.match_token(vec![RXTT::Or]) {
            let operator = self.previous().clone();
            let right = self.equality()?;

            expr = Logical::new(Rc::clone(&expr), operator, Rc::clone(&right));
        }

        Ok(expr)
    }

    fn while_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let condition = self.expression()?;
        self.consume(RXTT::RightParen)?;

        let body = self.statement()?;

        Ok(While::new(Rc::clone(&condition), Rc::clone(&body)))
    }

    fn for_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let mut initialiser = Rc::new(Stmt::Empty);

        if !self.match_token(vec![RXTT::Semicolon]) {
            if self.match_token(vec![RXTT::Var]) {
                initialiser = self.var_declaration()?;
            } else {
                initialiser = self.expression_statement()?;
            }
        }

        let mut condition = Literal::new(Object::Bool(true));

        if !self.check(&RXTT::Semicolon) {
            condition = self.expression()?;
        }

        self.consume(RXTT::Semicolon)?;

        let mut increment = Rc::new(Expr::Empty);

        if !self.match_token(vec![RXTT::RightParen]) {
            increment = self.expression()?;
        }

        self.consume(RXTT::RightParen)?;

        let mut body = self.statement()?;

        match increment.borrow() {
            Expr::Empty => {}
            _ => {
                body = Block::new(vec![body, Expression::new(Rc::clone(&increment))]);
            }
        }

        body = While::new(Rc::clone(&condition), Rc::clone(&body));

        match initialiser.borrow() {
            Stmt::Empty => {}
            _ => body = Block::new(vec![initialiser, body]),
        }

        Ok(body)
    }

    fn finish_call(&mut self, callee: &Rc<Expr>) -> Result<Rc<Expr>, RatexError> {
        let mut arguments = Vec::new();

        if !self.check(&RXTT::RightParen) {
            arguments.push(Rc::clone(&self.expression()?));

            while self.match_token(vec![RXTT::Comma]) {
                arguments.push(Rc::clone(&self.expression()?));
            }
        }

        let paren = self.consume(RXTT::RightParen)?;

        Ok(Call::new(Rc::clone(callee), paren.clone(), arguments))
    }

    fn function_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        let name = self.consume(RXTT::Identifier)?.clone();

        self.consume(RXTT::LeftParen)?;
        let mut params = Vec::new();

        if !self.check(&RXTT::RightParen) {
            params.push(self.consume(RXTT::Identifier)?.clone());

            while self.match_token(vec![RXTT::Comma]) {
                params.push(self.consume(RXTT::Identifier)?.clone());
            }
        }

        self.consume(RXTT::RightParen)?;
        self.consume(RXTT::LeftBrace)?;
        let body = self.block()?;

        Ok(Fun::new(name, params, body))
    }

    fn return_statement(&mut self) -> Result<Rc<Stmt>, RatexError> {
        let keyword = self.previous().clone();
        let mut value = Rc::new(Expr::Empty);
        if !self.check(&RXTT::Semicolon) {
            value = self.expression()?;
        }

        self.consume(RXTT::Semicolon)?;

        Ok(Return::new(keyword, Rc::clone(&value)))
    }

    fn anonymous_function(&mut self) -> Result<Rc<Expr>, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let mut params = Vec::new();

        if !self.check(&RXTT::RightParen) {
            params.push(self.consume(RXTT::Identifier)?.clone());

            while self.match_token(vec![RXTT::Comma]) {
                params.push(self.consume(RXTT::Identifier)?.clone());
            }
        }

        self.consume(RXTT::RightParen)?;
        self.consume(RXTT::LeftBrace)?;
        let body = self.block()?;

        Ok(Lambda::new(params, body))
    }

    fn class_declaration(&mut self) -> Result<Rc<Stmt>, RatexError> {
        let name = self.consume(RXTT::Identifier)?.clone();
        self.consume(RXTT::LeftBrace)?;

        let mut methods = Vec::new();

        while !self.check(&RXTT::RightBrace) && !self.is_at_end() {
            methods.push(Rc::clone(&self.function_statement()?));
        }

        self.consume(RXTT::RightBrace)?;

        Ok(Class::new(name, methods))
    }
}
