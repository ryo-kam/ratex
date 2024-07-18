use crate::{
    ast::{Binary, Expr, Grouping, Literal, LiteralValue, Unary},
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

    pub fn parse(&mut self) -> Result<Expr, RatexError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expr, RatexError> {
        self.equality()
    }

    fn equality(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.comparison()?;

        while self.match_token(vec![RXTT::BangEqual, RXTT::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }));
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
            expr = Expr::Binary(Box::new(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.factor()?;

        while self.match_token(vec![RXTT::Minus, RXTT::Plus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, RatexError> {
        let mut expr = self.unary()?;

        while self.match_token(vec![RXTT::Slash, RXTT::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            }));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, RatexError> {
        if self.match_token(vec![RXTT::Bang, RXTT::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary(Box::new(Unary {
                operator,
                right: Box::new(right),
            })))
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, RatexError> {
        match &self.tokens.get(self.current).unwrap().token {
            RXTT::False => {
                self.current += 1;
                Ok(Expr::Literal(Box::new(Literal {
                    value: LiteralValue::Bool(false),
                })))
            }
            RXTT::True => {
                self.current += 1;
                Ok(Expr::Literal(Box::new(Literal {
                    value: LiteralValue::Bool(true),
                })))
            }
            RXTT::Nil => {
                self.current += 1;
                Ok(Expr::Literal(Box::new(Literal {
                    value: LiteralValue::Nil,
                })))
            }
            RXTT::Number(n) => {
                self.current += 1;
                Ok(Expr::Literal(Box::new(Literal {
                    value: LiteralValue::Number(n.clone()),
                })))
            }
            RXTT::String(s) => {
                self.current += 1;
                Ok(Expr::Literal(Box::new(Literal {
                    value: LiteralValue::String(s.clone()),
                })))
            }
            RXTT::LeftParen => {
                self.current += 1;
                let expr: Expr = self.expression().unwrap();

                let mut ind = self.current;

                loop {
                    match self.tokens.get(ind).unwrap().token {
                        RXTT::RightParen => {
                            break;
                        }
                        RXTT::EOF => {
                            return Err(RatexError {
                                source: RatexErrorType::UnterminatedParen(self.peek().line),
                            });
                        }
                        _ => {
                            ind += 1;
                        }
                    }
                }

                Ok(Expr::Grouping(Box::new(Grouping {
                    expr: Box::new(expr),
                })))
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

    fn is_at_end(&self) -> bool {
        self.peek().token == RXTT::EOF
    }
}
