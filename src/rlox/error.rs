use std::{io, rc::Rc, result};

use super::{token::Token, types::TokenType};

pub type Result<T, E = LoxError> = result::Result<T, E>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LoxError {
    IoError {
        msg: String,
    },
    ParseTokenError {
        position: (usize, usize),
        msg: &'static str,
    },
    ParseError {
        position: (usize, usize),
        lexeme: Rc<String>,
        token_type: TokenType,
        msg: String,
    },
    RuntimeError {
        position: (usize, usize),
        lexeme: Rc<String>,
        msg: String,
    },
    UnexpectedError {
        message: String,
    },
}

impl LoxError {
    pub fn create_runtime_error(token: &Token, msg: String) -> Self {
        Self::RuntimeError {
            position: token.position,
            lexeme: token.lexeme.clone(),
            msg,
        }
    }
}

impl From<io::Error> for LoxError {
    fn from(value: io::Error) -> Self {
        Self::IoError {
            msg: value.to_string(),
        }
    }
}
