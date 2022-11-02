use std::vec;

use super::error::LoxError::ParseError;
use super::error::{LoxError, Result};
use super::expr::Expression;
use super::stmt::{self, Statement};
use super::token::{Literal, Token};
use super::token_type::TokenType;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Statement>> {
        let mut statements = vec![];

        while !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expression> {
        self.assignment()
    }

    fn declaration(&mut self) -> Result<Statement> {
        if self.matched(vec![TokenType::Let]) {
            return self.var_declaration();
        }

        match self.statement() {
            Ok(stmt) => Ok(stmt),
            Err(err) => {
                self.synchronize();
                Err(err)
            }
        }
    }

    fn var_declaration(&mut self) -> Result<Statement> {
        let mut vars = vec![];

        while !self.is_at_end() && !self.check(TokenType::Semicolon) {
            let name = self.consume(TokenType::Identifier, "Expect a variable name")?;
            let mut initializer = None;
            if self.matched(vec![TokenType::Equal]) {
                initializer = Some(self.ternary()?)
            }
            vars.push(stmt::VarStatement::new(name, initializer));
            if !self.check(TokenType::Semicolon) {
                self.consume(TokenType::Comma, "Expect ',' after value")?;
            }
        }

        self.consume(TokenType::Semicolon, "Expect ';' after value")?;

        Ok(Statement::create_multi_var_statement(vars))
    }

    fn statement(&mut self) -> Result<Statement> {
        if self.matched(vec![TokenType::For]) {
            return self.for_statement();
        }

        if self.matched(vec![TokenType::If]) {
            return self.branch_statement();
        }

        if self.matched(vec![TokenType::Print]) {
            return self.print_statement();
        }

        if self.matched(vec![TokenType::While]) {
            return self.while_statement();
        }

        if self.matched(vec![TokenType::Break]) {
            let token = self.previous();
            self.consume(TokenType::Semicolon, "Expect ';' after 'break'")?;
            return Ok(Statement::create_break_statement(token));
        }

        if self.matched(vec![TokenType::Continue]) {
            let token = self.previous();
            self.consume(TokenType::Semicolon, "Expect ';' after 'continue'")?;
            return Ok(Statement::create_continue_statement(token));
        }

        if self.matched(vec![TokenType::LeftBrace]) {
            return Ok(Statement::create_block_statement(self.block_statement()?));
        }

        self.expression_statement()
    }

    fn for_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'")?;

        let initializer = if self.check(TokenType::Let) {
            self.advance();
            Some(self.var_declaration()?)
        } else if self.check(TokenType::Semicolon) {
            self.advance();
            None
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if self.check(TokenType::Semicolon) {
            None
        } else {
            Some(self.expression()?)
        };

        self.consume(TokenType::Semicolon, "Expect ';' after loop condition")?;

        let increment = if self.check(TokenType::RightParen) {
            None
        } else {
            Some(Statement::create_expression_statement(self.expression()?))
        };

        self.consume(TokenType::RightParen, "Expect ')' after 'for'")?;

        let mut body = self.statement()?;

        let mut incr = None;

        if let Some(inc) = increment {
            body = Statement::create_block_statement(vec![body, inc.clone()]);
            incr = Some(Box::new(inc))
        }

        body = Statement::create_while_statement(
            condition.unwrap_or_else(|| Expression::create_literal_expression(Literal::Bool(true))),
            Box::new(body),
            incr
        );

        if let Some(init) = initializer {
            body = Statement::create_block_statement(vec![init, body]);
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after the condition")?;
        let body = self.statement()?;

        Ok(Statement::create_while_statement(condition, Box::new(body), None))
    }

    fn branch_statement(&mut self) -> Result<Statement> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after the condition")?;

        let then_branch = self.statement()?;
        let mut else_branch = None;

        if self.matched(vec![TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()?));
        }

        Ok(Statement::create_branch_statement(
            condition,
            Box::new(then_branch),
            else_branch,
        ))
    }

    fn block_statement(&mut self) -> Result<Vec<Statement>> {
        let mut statements = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?)
        }

        self.consume(TokenType::RightBrace, "Expect '}' after block")?;

        Ok(statements)
    }

    fn print_statement(&mut self) -> Result<Statement> {
        let value = self.expression()?;

        self.consume(TokenType::Semicolon, "Expect ';' after value")?;

        Ok(Statement::create_print_statement(value))
    }

    fn expression_statement(&mut self) -> Result<Statement> {
        let expr = self.expression()?;

        self.consume(TokenType::Semicolon, "Expect ';' after value")?;

        Ok(Statement::create_expression_statement(expr))
    }

    fn assignment(&mut self) -> Result<Expression> {
        let expr = self.or()?;

        if self.matched(vec![TokenType::Equal]) {
            let eq = self.previous();
            let value = self.assignment()?;

            if let Expression::VariableExpression(e) = expr {
                let name = e.name;
                return Ok(Expression::create_assign_expression(name, Box::new(value)));
            }

            return Err(LoxError::create_runtime_error(
                &eq,
                "Invalid assignment target".into(),
            ));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expression> {
        let mut expr = self.and()?;

        while self.matched(vec![TokenType::Or]) && !self.is_at_end() {
            let op = self.previous();
            let right = self.and()?;
            expr = Expression::create_logical_expression(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expression> {
        let mut expr = self.comma()?;

        while self.matched(vec![TokenType::And]) && !self.is_at_end() {
            let op = self.previous();
            let right = self.comma()?;
            expr = Expression::create_logical_expression(Box::new(expr), op, Box::new(right));
        }

        Ok(expr)
    }

    fn comma(&mut self) -> Result<Expression> {
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

    fn ternary(&mut self) -> Result<Expression> {
        let cmp = self.equality();

        if self.matched(vec![TokenType::QuestionMark]) {
            let true_value = self.ternary();
            return if self.matched(vec![TokenType::Colon]) {
                let false_value = self.ternary();
                Ok(Expression::create_ternary_expression(
                    Box::new(cmp?),
                    Box::new(true_value?),
                    Box::new(false_value?),
                ))
            } else {
                Err(Self::error(self.peek(), "Expected ':' after expression"))
            };
        };

        cmp
    }

    fn equality(&mut self) -> Result<Expression> {
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

    fn comparison(&mut self) -> Result<Expression> {
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

    fn term(&mut self) -> Result<Expression> {
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

    fn factor(&mut self) -> Result<Expression> {
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

    fn unary(&mut self) -> Result<Expression> {
        if self.matched(vec![TokenType::Bang, TokenType::Minus, TokenType::Plus]) {
            let op = self.previous();
            let right = self.unary();
            return Ok(Expression::create_unary_expression(op, Box::new(right?)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expression> {
        if self.matched(vec![TokenType::False]) {
            Ok(Expression::create_literal_expression(Literal::Bool(false)))
        } else if self.matched(vec![TokenType::True]) {
            Ok(Expression::create_literal_expression(Literal::Bool(true)))
        } else if self.matched(vec![TokenType::Nil]) {
            Ok(Expression::create_literal_expression(Literal::Nil))
        } else if self.matched(vec![TokenType::Number, TokenType::String]) {
            Ok(Expression::create_literal_expression(
                self.previous().literal.unwrap(),
            ))
        } else if self.matched(vec![TokenType::Identifier]) {
            Ok(Expression::create_variable_expression(self.previous()))
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

    fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token> {
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
        ParseError {
            line: token.line,
            lexeme: token.lexeme.clone(),
            token_type: token.token_type,
            msg: msg.into(),
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }
            use TokenType::{Class, For, Fun, If, Let, Print, Return, While};

            match self.previous().token_type {
                Class | Fun | Let | For | If | While | Print | Return => {
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
