use std::{io, result, rc::Rc};

use super::{
    token::Token,
    types::{Literal, TokenType},
};

pub type Result<T, E = LoxError> = result::Result<T, E>;

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum LoxError {
    IoError {
        msg: String,
    },
    ParseTokenError {
        line: usize,
        msg: &'static str,
    },
    ParseError {
        line: usize,
        lexeme: Rc<String>,
        token_type: TokenType,
        msg: String,
    },
    RuntimeError {
        line: usize,
        lexeme: Rc<String>,
        msg: String,
    },
    Break {
        line: usize,
        lexeme: Rc<String>,
        msg: String,
    },
    Continue {
        line: usize,
        lexeme: Rc<String>,
        msg: String,
    },
    Return {
        value: Literal,
    },
}

impl LoxError {
    pub fn create_runtime_error(token: &Token, msg: String) -> Self {
        Self::RuntimeError {
            line: token.line,
            lexeme: token.lexeme.clone(),
            msg,
        }
    }

    pub fn create_break(token: &Token, msg: String) -> Self {
        Self::Break {
            line: token.line,
            lexeme: token.lexeme.clone(),
            msg,
        }
    }

    pub fn create_continue(token: &Token, msg: String) -> Self {
        Self::Continue {
            line: token.line,
            lexeme: token.lexeme.clone(),
            msg,
        }
    }

    pub fn create_return(value: Literal) -> Self {
        Self::Return { value }
    }
}

impl From<io::Error> for LoxError {
    fn from(value: io::Error) -> Self {
        Self::IoError {
            msg: value.to_string(),
        }
    }
}
