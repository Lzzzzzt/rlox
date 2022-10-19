use crate::lib::error::LoxError;
use crate::lib::error::LoxError::ParseError;
use crate::lib::expr::Expression;
use crate::lib::token::{Literal, Token};
use crate::lib::token_type::TokenType;
use crate::Lox;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Expression, LoxError> {
        self.expression()
    }

    fn expression(&mut self) -> Result<Expression, LoxError> {
        self.comma()
    }

    fn comma(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.ternary();

        while self.matched(vec![TokenType::Comma]) {
            let op = self.previous();
            let right = self.ternary();
            expr = Ok(Expression::create_binary_expression(
                Box::new(expr?),
                op,
                Box::new(right?),
            ));
        }

        expr
    }

    fn ternary(&mut self) -> Result<Expression, LoxError> {
        let cmp = self.equality();

        if self.matched(vec![TokenType::QuestionMark]) {
            let true_value = self.ternary();
            return if self.matched(vec![TokenType::Colon]) {
                let false_value = self.ternary();
                Ok(Expression::create_ternary_expression(Box::new(cmp?), Box::new(true_value?), Box::new(false_value?)))
            } else {
                Err(Self::error(self.peek(), "Expected ':' after expression"))
            };
        };

        cmp
    }

    fn equality(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.comparison();

        while self.matched(vec![TokenType::BangEqual, TokenType::EqualEqual]) {
            let op = self.previous();
            let right = self.comparison();
            expr = Ok(Expression::create_binary_expression(
                Box::new(expr?),
                op,
                Box::new(right?),
            ));
        }

        expr
    }

    fn comparison(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.term();

        while self.matched(vec![
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let op = self.previous();
            let right = self.term();
            expr = Ok(Expression::create_binary_expression(
                Box::new(expr?),
                op,
                Box::new(right?),
            ));
        }

        expr
    }

    fn term(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.factor();

        while self.matched(vec![TokenType::Plus, TokenType::Minus]) {
            let op = self.previous();
            let right = self.factor();
            expr = Ok(Expression::create_binary_expression(
                Box::new(expr?),
                op,
                Box::new(right?),
            ));
        }

        expr
    }

    fn factor(&mut self) -> Result<Expression, LoxError> {
        let mut expr = self.unary();

        while self.matched(vec![TokenType::Star, TokenType::Slash]) {
            let op = self.previous();
            let right = self.unary();
            expr = Ok(Expression::create_binary_expression(
                Box::new(expr?),
                op,
                Box::new(right?),
            ));
        }

        expr
    }

    fn unary(&mut self) -> Result<Expression, LoxError> {
        if self.matched(vec![TokenType::Bang, TokenType::Minus]) {
            let op = self.previous();
            let right = self.unary();
            return Ok(Expression::create_unary_expression(op, Box::new(right?)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression, LoxError> {
        if self.matched(vec![TokenType::False]) {
            Ok(Expression::create_literal_expression(Literal::False))
        } else if self.matched(vec![TokenType::True]) {
            Ok(Expression::create_literal_expression(Literal::True))
        } else if self.matched(vec![TokenType::Nil]) {
            Ok(Expression::create_literal_expression(Literal::Nil))
        } else if self.matched(vec![TokenType::Number, TokenType::String]) {
            Ok(Expression::create_literal_expression(self.previous().literal.unwrap()))
        } else if self.matched(vec![TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.")
                .unwrap();
            Ok(Expression::create_grouping_expression(Box::new(expr?)))
        } else {
            use TokenType::{
                BangEqual, Comma, EqualEqual, Greater, GreaterEqual, Less, LessEqual, Slash, Star,
            };
            let token = self.peek();

            match token.token_type {
                Star | Slash | Comma | Greater | GreaterEqual | Less | LessEqual | EqualEqual
                | BangEqual => Err(Self::error(
                    token,
                    format!("Expect a expression before '{}'", token.lexeme).as_str(),
                )),
                _ => Err(Self::error(token, "Expect expression.")),
            }
        }
    }

    fn matched(&mut self, types: Vec<TokenType>) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }

        false
    }

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token, LoxError> {
        if self.check(token_type) {
            return Ok(self.advance());
        }

        Err(Self::error(self.peek(), msg))
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type == token_type
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap()
    }

    fn previous(&self) -> Token {
        (*self.tokens.get(self.current - 1).unwrap()).clone()
    }

    fn error(token: &Token, msg: &str) -> LoxError {
        Lox::token_error(token, msg);
        ParseError
    }

    #[allow(unused)]
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            use TokenType::{Class, For, Fun, If, Print, Return, Var, While};

            match self.previous().token_type {
                Class | Fun | Var | For | If | While | Print | Return => {
                    return;
                }
                _ => {
                    self.advance();
                }
            }
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }
}
