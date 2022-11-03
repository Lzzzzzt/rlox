use crate::rlox::token::{Literal, Token, KEYWORD_MAP};
use crate::rlox::token_type::TokenType;

use super::error::LoxError;

pub struct Scanner {
    source: String,
    pub tokens: Vec<Token>,

    start: usize,
    current: usize,
    line: usize,
}

impl Scanner {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: vec![],

            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<(), LoxError> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::Eof, "".into(), self.line));
        // return self.tokens;

        Ok(())
    }

    fn scan_token(&mut self) -> Result<(), LoxError> {
        let cur = self.advance();

        match cur {
            '?' => self.add_token(TokenType::QuestionMark),
            ':' => self.add_token(TokenType::Colon),
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '+' => self.add_token(TokenType::Plus),
            '-' => self.add_token(TokenType::Minus),
            '*' => self.add_token(TokenType::Star),
            ';' => self.add_token(TokenType::Semicolon),
            '!' => {
                let token = if self.expected('=') {
                    self.advance();
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.expected('=') {
                    self.advance();
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.expected('=') {
                    self.advance();
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.expected('=') {
                    self.advance();
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };

                self.add_token(token);
            }
            '/' => {
                if self.expected('/') {
                    while self.nth(0) != '\n' || !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.parse_string()?,
            _ => {
                if cur.is_ascii_digit() {
                    self.parse_number();
                } else if cur.is_ascii_alphabetic() || cur == '_' {
                    self.parse_identifier();
                } else {
                    return Err(LoxError::ParseTokenError {
                        line: self.line,
                        msg: "Unexpected character.",
                    });
                }
            }
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<(), LoxError> {
        while self.nth(0) != '"' && !self.is_at_end() {
            if self.nth(0) == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(LoxError::ParseTokenError {
                line: self.line,
                msg: "Unterminated String.",
            });
        }

        self.advance();

        self.add_token_with_literal(
            TokenType::String,
            Literal::String(self.source[self.start + 1..self.current - 1].into()),
        );

        Ok(())
    }

    fn parse_number(&mut self) {
        while self.nth(0).is_ascii_digit() {
            self.advance();
        }

        if self.nth(0) == '.' && self.nth(1).is_ascii_digit() {
            self.advance();

            while self.nth(0).is_ascii_digit() {
                self.advance();
            }
        }
        self.add_token_with_literal(
            TokenType::Number,
            Literal::Number(self.source[self.start..self.current].parse().unwrap()),
        );
    }

    fn parse_identifier(&mut self) {
        while self.nth(0).is_ascii_alphanumeric() || self.nth(0) == '_' {
            self.advance();
        }
        let text = &self.source[self.start..self.current];

        let token_type = KEYWORD_MAP.get(text);

        match token_type {
            None => self.add_token(TokenType::Identifier),
            Some(token_type) => self.add_token(*token_type),
        }
    }

    #[inline]
    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn nth(&self, n: usize) -> char {
        if self.current + n >= self.source.len() {
            return '\0';
        }
        self.source.chars().nth(self.current + n).unwrap()
    }

    fn advance(&mut self) -> char {
        self.current += 1;
        return self.source.chars().nth(self.current - 1).unwrap();
    }

    fn expected(&self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }

        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        true
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = &self.source[self.start..self.current];
        self.tokens
            .push(Token::new(token_type, text.into(), self.line));
    }

    fn add_token_with_literal(&mut self, token_type: TokenType, literal: Literal) {
        let text = &self.source[self.start..self.current];
        self.tokens.push(Token::with_literal(
            token_type,
            text.into(),
            Some(literal),
            self.line,
        ));
    }
}