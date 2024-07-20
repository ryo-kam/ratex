use crate::{
    ast::{
        Binary, Expr, Expression, Grouping, Literal, LiteralValue, Print, Stmt, Unary, Var,
        Variable,
    },
    error::{RatexError, RatexErrorType},
    token::{RatexToken as RXT, RatexTokenType as RXTT},
};

pub struct Parser {
    tokens: Vec<RXT>,
    current: usize,
}

impl Parser {
    pub fn new(input: Vec<RXT>) -> Self {
        Parser {
            tokens: input,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, RatexError> {
        let mut statements: Vec<Stmt> = Vec::new();

        while !self.is_at_end() {
            statements.push(self.declaration());
        }

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, RatexError> {
        self.equality()
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
                let expr: Expr = self.expression().unwrap();

                self.consume(
                    RXTT::RightParen,
                    "Expected ')' after parenthesis.".to_string(),
                );

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
                    format!("Unexpected token: {}", self.peek().lexeme),
                ),
            }),
        }
    }

    fn match_token(&mut self, vec: Vec<RXTT>) -> bool {
        for token_type in vec {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn previous(&self) -> &RXT {
        self.tokens.get(self.current - 1).unwrap()
    }

    fn check(&self, token_type: RXTT) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token == token_type
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

    fn consume(&mut self, token_type: RXTT, error: String) -> &RXT {
        if self.check(token_type) {
            return self.advance();
        } else {
            panic!("{error}")
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token == RXTT::EOF
    }

    fn statement(&mut self) -> Stmt {
        if self.match_token(vec![RXTT::Print]) {
            return self.print_statement();
        }

        self.expression_statement()
    }

    fn print_statement(&mut self) -> Stmt {
        let value = self.expression().unwrap();

        self.consume(RXTT::Semicolon, "Expected ';' after value.".to_string());

        return Stmt::Print(Print {
            expr: Box::new(value),
        });
    }

    fn expression_statement(&mut self) -> Stmt {
        let value = self.expression().unwrap();

        self.consume(RXTT::Semicolon, "Expected ';' after value.".to_string());

        return Stmt::Expression(Expression {
            expr: Box::new(value),
        });
    }

    fn declaration(&mut self) -> Stmt {
        if self.match_token(vec![RXTT::Var]) {
            return self.var_declaration();
        } else {
            return self.statement();
        }
    }

    fn var_declaration(&mut self) -> Stmt {
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
            initialiser = self.expression().unwrap();
        }

        self.consume(
            RXTT::Semicolon,
            "Expected ';' after variable declaration".to_owned(),
        );

        return Stmt::Var(Var {
            name,
            initialiser: Box::new(initialiser),
        });
    }
}
