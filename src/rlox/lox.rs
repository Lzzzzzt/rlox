use std::fs::read_to_string;

use std::path::PathBuf;
use std::time::SystemTime;

use super::parser::Parser;
use super::repl;
use super::scanner::Scanner;
use super::token::Token;
use super::types::TokenType;

use super::error::LoxError;

use super::interpreter::Interpreter;

static mut HAD_ERROR: bool = false;

pub fn is_error() -> bool {
    unsafe { HAD_ERROR }
}

pub fn no_error() {
    unsafe { HAD_ERROR = false }
}

pub fn had_error() {
    unsafe { HAD_ERROR = true }
}

pub struct Lox;

impl Lox {
    pub fn run_file(path: PathBuf) -> Result<(), LoxError> {
        let string = read_to_string(path)?;

        let mut scanner = Scanner::new(string);

        if let Err(err) = scanner.scan_tokens() {
            Self::error(err);
            had_error();
        }

        let mut interpreter = Interpreter::new();

        Self::run(&mut interpreter, scanner.tokens);

        if is_error() {
            eprintln!("Exit because error before!");
        }

        Ok(())
    }

    pub fn run_prompt() -> Result<(), LoxError> {
        let mut repl = repl::Repl::new();
        repl.run(Self::run);
        Ok(())
    }

    fn run(interpreter: &mut Interpreter, tokens: Vec<Token>) {
        let start = SystemTime::now();

        let mut parser = Parser::new(tokens);

        match parser.parse() {
            Ok(expression) => match interpreter.interpret(&expression) {
                Ok(value) => value,
                Err(err) => Self::error(err),
            },
            Err(err) => {
                for e in err {
                    Self::error(e)
                }
            }
        }

        println!(
            "Total Cost {}ms",
            SystemTime::now().duration_since(start).unwrap().as_micros() as f64 / 1000.0
        );
    }

    pub fn error(error: LoxError) {
        match error {
            LoxError::ParseError {
                position: line,
                lexeme,
                msg,
                token_type,
            } => {
                if token_type == TokenType::Eof {
                    Self::report(line, "at end", msg.as_str())
                } else {
                    Self::report(line, format!("at `{}`", lexeme).as_str(), msg.as_str())
                }
            }
            LoxError::RuntimeError {
                position,
                lexeme,
                msg,
            }
            | LoxError::Break {
                position,
                lexeme,
                msg,
            }
            | LoxError::Continue {
                position,
                lexeme,
                msg,
            } => Self::report(position, format!("at `{}`", lexeme).as_str(), msg.as_str()),
            LoxError::IoError { msg } => Self::report((0, 0), "", msg.as_str()),
            LoxError::ParseTokenError {
                position: line,
                msg,
            } => Self::report(line, "", msg),
            LoxError::Return { .. } => (),
        }
    }

    fn report(line: (usize, usize), err_pos: &str, msg: &str) {
        let err_msg = if err_pos.is_empty() {
            if line != (0, 0) {
                format!("[row:{:3}, col:{:3}] LoxError: {msg}", line.0, line.1)
            } else {
                format!("[----------------] LoxError: {msg}")
            }
        } else if line != (0, 0) {
            format!(
                "[row:{:3}, col:{:3}] LoxError {err_pos}: {msg}",
                line.0, line.1
            )
        } else {
            format!("[----------------] LoxError {err_pos}: {msg}")
        };

        println!("\x1b[1;31m{err_msg}\x1b[0m");
        had_error()
    }
}
