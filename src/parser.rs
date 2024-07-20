use crate::{
    ast::{
        Assign, Binary, Block, Expr, Expression, Grouping, If, Literal, LiteralValue, Logical,
        Print, Stmt, Unary, Var, Variable, While,
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
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, RatexError> {
        match &self.tokens.get(self.current).unwrap().token {
            RXTT::False => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: LiteralValue::Bool(false),
                }))
            }
            RXTT::True => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: LiteralValue::Bool(true),
                }))
            }
            RXTT::Nil => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: LiteralValue::Nil,
                }))
            }
            RXTT::Number(n) => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: LiteralValue::Number(n.clone()),
                }))
            }
            RXTT::String(s) => {
                self.current += 1;
                Ok(Expr::Literal(Literal {
                    value: LiteralValue::String(s.clone()),
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
            RXTT::Identifier(_) => {
                self.current += 1;
                Ok(Expr::Variable(Variable {
                    name: self.previous().clone(),
                }))
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

        self.peek().token == *token_type
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
        self.peek().token == RXTT::EOF
    }

    fn statement(&mut self) -> Result<Stmt, RatexError> {
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

    fn declaration(&mut self) -> Result<Stmt, RatexError> {
        if self.match_token(vec![RXTT::Var]) {
            Ok(self.var_declaration()?)
        } else {
            Ok(self.statement()?)
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, RatexError> {
        let name = match &self.peek().token {
            RXTT::Identifier(s) => RXT {
                token: RXTT::Identifier(s.clone()),
                lexeme: s.clone(),
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
            match self.previous().token {
                RXTT::Semicolon => return (),
                _ => {}
            }

            match self.peek().token {
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
            statements.push(self.declaration()?);
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
}
