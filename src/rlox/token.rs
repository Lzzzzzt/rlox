use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

use lazy_static::lazy_static;

use super::types::Literal;
use super::types::TokenType;

lazy_static! {
    pub static ref KEYWORD_MAP: HashMap<&'static str, TokenType> = HashMap::from_iter([
        ("and", TokenType::And),
        ("class", TokenType::Class),
        ("else", TokenType::Else),
        ("false", TokenType::False),
        ("for", TokenType::For),
        ("func", TokenType::Func),
        ("if", TokenType::If),
        ("nil", TokenType::Nil),
        ("or", TokenType::Or),
        ("print", TokenType::Print),
        ("return", TokenType::Return),
        ("super", TokenType::Super),
        ("self", TokenType::RSelf),
        ("true", TokenType::True),
        ("let", TokenType::Let),
        ("while", TokenType::While),
        ("continue", TokenType::Continue),
        ("break", TokenType::Break),
        ("#[static]", TokenType::Static),
        ("extend", TokenType::Extend)
    ]);
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: Rc<String>,
    pub literal: Option<Literal>,
    pub position: (usize, usize),
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, position: (usize, usize)) -> Self {
        Self {
            token_type,
            lexeme: Rc::new(lexeme),
            position,
            literal: None,
        }
    }

    pub fn with_literal(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        position: (usize, usize),
    ) -> Self {
        Self {
            token_type,
            lexeme: Rc::new(lexeme),
            literal,
            position,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}
