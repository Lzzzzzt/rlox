use std::collections::HashMap;
use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;

use crate::lib::token_type::TokenType;

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number(f32),
}

lazy_static! {
    pub static ref KEYWORD_MAP: HashMap<&'static str, TokenType> = HashMap::from_iter([
    ("and", TokenType::And),
    ("class", TokenType::Class),
    ("else", TokenType::Else),
    ("false", TokenType::False),
    ("for", TokenType::For),
    ("fun", TokenType::Fun),
    ("if", TokenType::If),
    ("nil", TokenType::Nil),
    ("or", TokenType::Or),
    ("print", TokenType::Print),
    ("return", TokenType::Return),
    ("super", TokenType::Super),
    ("this", TokenType::This),
    ("true", TokenType::True),
    ("var", TokenType::Var),
    ("while", TokenType::While),
]);
}

pub struct Token {
    token_type: TokenType,
    lexeme: String,
    // Todo: final Object literal;
    literal: Option<Literal>,
    #[allow(unused)]
    line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
            literal: None,
        }
    }

    pub fn with_literal(token_type: TokenType, lexeme: String, literal: Option<Literal>, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            line,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.literal {
            None => {
                write!(f, "{:?} {}", self.token_type, self.lexeme)
            }
            Some(_) => {
                write!(f, "{:?} {} {:?}", self.token_type, self.lexeme, self.literal.as_ref().unwrap())
            }
        }
    }
}