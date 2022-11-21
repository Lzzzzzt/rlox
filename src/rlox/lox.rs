use std::fs::read_to_string;

use std::path::PathBuf;
use std::time::SystemTime;

use crate::rlox::bytecode_interpreter::vm::VirtualMachine;

use super::bytecode_interpreter::convertor::Convertor;
use super::parser::Parser;
use super::repl;
use super::resolver::Resolver;
use super::scanner::Scanner;
use super::token::Token;
use super::types::TokenType;

use super::error::LoxError;

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
        std::env::set_var("RLOX_RUN_MODE", "F");

        let string = read_to_string(path)?;

        let mut scanner = Scanner::new(string);

        if let Err(err) = scanner.scan_tokens() {
            Self::error(err);
            had_error();
        }

        let mut vm = VirtualMachine::new();

        Self::run(&mut vm, scanner.tokens);

        if is_error() {
            eprintln!("Exit because error before!");
        }

        Ok(())
    }

    pub fn run_prompt() -> Result<(), LoxError> {
        std::env::set_var("RLOX_RUN_MODE", "R");
        let mut repl = repl::Repl::new();
        repl.run(Self::run);
        Ok(())
    }

    #[allow(unused)]
    fn run(vm: &mut VirtualMachine, tokens: Vec<Token>) {
        let start = SystemTime::now();

        let mut parser = Parser::new(tokens);
        let mut resolver = Resolver::new();

        match parser.parse() {
            Ok(statements) => match resolver.resolve(&statements) {
                Ok(_) => {
                    let mut convertor = Convertor::default();
                    match convertor.convert(&statements) {
                        Ok(func) => match vm.interpret(func) {
                            Ok(value) => value,
                            Err(err) => Self::error(err),
                        },
                        Err(err) => Self::error(err),
                    };
                }
                Err(err) => Self::error(err),
            },
            Err(err) => {
                for e in err {
                    Self::error(e)
                }
            }
        }

        if std::env::var("RLOX_RUN_MODE").unwrap() == "R" {
            println!(
                "\x1b[1;90m[TIME]: \x1b[0m{}ms",
                SystemTime::now().duration_since(start).unwrap().as_micros() as f64 / 1000.0
            );
        }
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
            } => Self::report(position, format!("at `{}`", lexeme).as_str(), msg.as_str()),
            LoxError::IoError { msg } => Self::report((0, 0), "", msg.as_str()),
            LoxError::ParseTokenError {
                position: line,
                msg,
            } => Self::report(line, "", msg),
            LoxError::UnexpectedError { message } => Self::report((0, 0), "", &message),
        }
    }

    fn report(position: (usize, usize), err_pos: &str, msg: &str) {
        let err_msg = if err_pos.is_empty() {
            if position != (0, 0) {
                format!("[{:2}, {:2}] LoxError: {msg}", position.0, position.1)
            } else {
                format!("[----------------] LoxError: {msg}")
            }
        } else if position != (0, 0) {
            format!(
                "[{:2},{:2}] LoxError {err_pos}: {msg}",
                position.0, position.1
            )
        } else {
            format!("[----------------] LoxError {err_pos}: {msg}")
        };

        println!("\x1b[1;31m{err_msg}\x1b[0m");
        had_error()
    }
}
