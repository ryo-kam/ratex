use crate::{
    ast::{
        Assign, Binary, Block, Break, Call, Class, Expr, Expression, Fun, Get, Grouping, If,
        Lambda, Literal, Logical, Object, Print, Return, Set, Stmt, Unary, Var, Variable, While,
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

    pub fn parse(&mut self) -> Vec<Stmt> {
        let mut statements: Vec<Stmt> = Vec::new();

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

    fn expression(&mut self) -> Result<Expr, RatexError> {
        self.assignment()
    }

    fn equality(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![RXTT::BangEqual, RXTT::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.term()?;

        while self.match_token(vec![
            RXTT::Greater,
            RXTT::GreaterEqual,
            RXTT::Less,
            RXTT::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.factor()?;

        while self.match_token(vec![RXTT::Minus, RXTT::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.unary()?;

        while self.match_token(vec![RXTT::Slash, RXTT::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, RatexError> {
        if self.match_token(vec![RXTT::Bang, RXTT::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary(Unary {
                operator,
                right: Box::new(right),
            }))
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(vec![RXTT::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(vec![RXTT::Dot]) {
                let name = self.consume(RXTT::Identifier)?;
                expr = Expr::Get(Get {
                    object: Box::new(expr),
                    name: name.clone(),
                })
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, RatexError> {
        match &self.tokens.get(self.current).unwrap().token_type {
            RXTT::False => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: Object::Bool(false),
                }))
            }
            RXTT::True => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: Object::Bool(true),
                }))
            }
            RXTT::Nil => {
                self.current += 1;
                Ok(Expr::Literal(Literal { value: Object::Nil }))
            }
            RXTT::Number(n) => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: Object::Number(n.clone()),
                }))
            }
            RXTT::String(s) => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: Object::String(s.clone()),
                }))
            }
            RXTT::LeftParen => {
                self.current += 1;
                let expr: Expr = self.expression()?;

                self.consume(RXTT::RightParen)?;

                Ok(Expr::Grouping(Grouping {
                    expr: Box::new(expr),
                }))
            }
            RXTT::Identifier => {
                self.current += 1;
                Ok(Expr::Variable(Variable {
                    name: self.previous().clone(),
                }))
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

    fn statement(&mut self) -> Result<Stmt, RatexError> {
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
            return Ok(Stmt::Block(Block {
                statements: self.block()?,
            }));
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Result<Stmt, RatexError> {
        let value = self.expression()?;

        self.consume(RXTT::Semicolon)?;

        Ok(Stmt::Print(Print {
            expr: Box::new(value),
        }))
    }

    fn expression_statement(&mut self) -> Result<Stmt, RatexError> {
        let value = self.expression()?;

        self.consume(RXTT::Semicolon)?;

        Ok(Stmt::Expression(Expression {
            expr: Box::new(value),
        }))
    }

    fn break_statement(&mut self) -> Result<Stmt, RatexError> {
        if self.match_token(vec![RXTT::Break]) {
            self.consume(RXTT::Semicolon)?;
            return Ok(Stmt::Break(Break {}));
        }

        self.declaration()
    }

    fn declaration(&mut self) -> Result<Stmt, RatexError> {
        if self.match_token(vec![RXTT::Var]) {
            Ok(self.var_declaration()?)
        } else {
            Ok(self.statement()?)
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, RatexError> {
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

        let mut initialiser: Expr = Expr::Empty;

        if self.match_token(vec![RXTT::Equal]) {
            initialiser = self.expression()?;
        }

        self.consume(RXTT::Semicolon)?;

        Ok(Stmt::Var(Var {
            name,
            initialiser: Box::new(initialiser),
        }))
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

    fn assignment(&mut self) -> Result<Expr, RatexError> {
        let expr = self.or()?;

        if self.match_token(vec![RXTT::Equal]) {
            let equals = self.previous();

            match expr {
                Expr::Variable(var) => {
                    let name = var.name;
                    let value = self.assignment()?;
                    return Ok(Expr::Assign(Assign {
                        name,
                        value: Box::new(value),
                    }));
                }
                Expr::Get(get) => {
                    return Ok(Expr::Set(Set {
                        object: get.object,
                        name: get.name,
                        value: Box::new(self.assignment()?),
                    }))
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

    fn block(&mut self) -> Result<Vec<Stmt>, RatexError> {
        let mut statements = Vec::new();

        while !self.check(&RXTT::RightBrace) && !self.is_at_end() {
            statements.push(self.break_statement()?);
        }

        self.consume(RXTT::RightBrace)?;

        Ok(statements)
    }

    fn if_statement(&mut self) -> Result<Stmt, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let condition = self.expression()?;
        self.consume(RXTT::RightParen)?;

        let then_stmt = self.statement()?;

        let mut else_stmt = Stmt::Empty;

        if self.match_token(vec![RXTT::Else]) {
            else_stmt = self.statement()?;
        }

        Ok(Stmt::If(If {
            condition: Box::new(condition),
            then_stmt: Box::new(then_stmt),
            else_stmt: Box::new(else_stmt),
        }))
    }

    fn or(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.and()?;

        while self.match_token(vec![RXTT::Or]) {
            let operator = self.previous().clone();
            let right = self.and()?;

            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            });
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.equality()?;

        while self.match_token(vec![RXTT::Or]) {
            let operator = self.previous().clone();
            let right = self.equality()?;

            expr = Expr::Logical(Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            })
        }

        Ok(expr)
    }

    fn while_statement(&mut self) -> Result<Stmt, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let condition = self.expression()?;
        self.consume(RXTT::RightParen)?;

        let body = self.statement()?;

        Ok(Stmt::While(While {
            condition: Box::new(condition),
            body: Box::new(body),
        }))
    }

    fn for_statement(&mut self) -> Result<Stmt, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let mut initialiser = Stmt::Empty;

        if !self.match_token(vec![RXTT::Semicolon]) {
            if self.match_token(vec![RXTT::Var]) {
                initialiser = self.var_declaration()?;
            } else {
                initialiser = self.expression_statement()?;
            }
        }

        let mut condition = Expr::Literal(Literal {
            value: Object::Bool(true),
        });

        if !self.check(&RXTT::Semicolon) {
            condition = self.expression()?;
        }

        self.consume(RXTT::Semicolon)?;

        let mut increment = Expr::Empty;

        if !self.match_token(vec![RXTT::RightParen]) {
            increment = self.expression()?;
        }

        self.consume(RXTT::RightParen)?;

        let mut body = self.statement()?;

        match increment {
            Expr::Empty => {}
            _ => {
                body = Stmt::Block(Block {
                    statements: vec![
                        body,
                        Stmt::Expression(Expression {
                            expr: Box::new(increment),
                        }),
                    ],
                });
            }
        }

        body = Stmt::While(While {
            condition: Box::new(condition),
            body: Box::new(body),
        });

        match initialiser {
            Stmt::Empty => {}
            _ => {
                body = Stmt::Block(Block {
                    statements: vec![initialiser, body],
                })
            }
        }

        Ok(body)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, RatexError> {
        let mut arguments = Vec::new();

        if !self.check(&RXTT::RightParen) {
            arguments.push(self.expression()?);

            while self.match_token(vec![RXTT::Comma]) {
                arguments.push(self.expression()?);
            }
        }

        let paren = self.consume(RXTT::RightParen)?;

        Ok(Expr::Call(Call {
            callee: Box::new(callee),
            paren: paren.clone(),
            arguments,
        }))
    }

    fn function_statement(&mut self) -> Result<Stmt, RatexError> {
        let name = self.consume(RXTT::Identifier)?.clone();

        self.consume(RXTT::LeftParen)?;
        let mut parameters = Vec::new();

        if !self.check(&RXTT::RightParen) {
            parameters.push(self.consume(RXTT::Identifier)?.clone());

            while self.match_token(vec![RXTT::Comma]) {
                parameters.push(self.consume(RXTT::Identifier)?.clone());
            }
        }

        self.consume(RXTT::RightParen)?;
        self.consume(RXTT::LeftBrace)?;
        let body = self.block()?;

        Ok(Stmt::Fun(Fun {
            name,
            params: parameters,
            body,
        }))
    }

    fn return_statement(&mut self) -> Result<Stmt, RatexError> {
        let keyword = self.previous().clone();
        let mut value = Expr::Empty;
        if !self.check(&RXTT::Semicolon) {
            value = self.expression()?;
        }

        self.consume(RXTT::Semicolon)?;

        Ok(Stmt::Return(Return {
            keyword,
            value: Box::new(value),
        }))
    }

    fn anonymous_function(&mut self) -> Result<Expr, RatexError> {
        self.consume(RXTT::LeftParen)?;
        let mut parameters = Vec::new();

        if !self.check(&RXTT::RightParen) {
            parameters.push(self.consume(RXTT::Identifier)?.clone());

            while self.match_token(vec![RXTT::Comma]) {
                parameters.push(self.consume(RXTT::Identifier)?.clone());
            }
        }

        self.consume(RXTT::RightParen)?;
        self.consume(RXTT::LeftBrace)?;
        let body = self.block()?;

        Ok(Expr::Lambda(Lambda {
            params: parameters,
            body,
        }))
    }

    fn class_declaration(&mut self) -> Result<Stmt, RatexError> {
        let name = self.consume(RXTT::Identifier)?.clone();
        self.consume(RXTT::LeftBrace)?;

        let mut methods = Vec::new();

        while !self.check(&RXTT::RightBrace) && !self.is_at_end() {
            methods.push(self.function_statement()?);
        }

        self.consume(RXTT::RightBrace)?;

        Ok(Stmt::Class(Class { name, methods }))
    }
}
