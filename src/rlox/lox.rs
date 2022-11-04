use std::env;
use std::fs::read_to_string;
use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::SystemTime;

use super::parser::Parser;
use super::scanner::Scanner;
use super::types::TokenType;

use super::error::LoxError;

use super::interpreter::Interpreter;

static mut HAD_ERROR: bool = false;

fn is_error() -> bool {
    unsafe { HAD_ERROR }
}

fn no_error() {
    unsafe { HAD_ERROR = false }
}

fn had_error() {
    unsafe { HAD_ERROR = true }
}

pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    pub fn run_file(mut self, path: PathBuf) -> Result<(), LoxError> {
        let string = read_to_string(path)?;
        self.run(string);

        if is_error() {
            eprintln!("Exit because error before!");
        }

        Ok(())
    }

    pub fn run_prompt(mut self) -> Result<(), LoxError> {
        let stdin = std::io::stdin();

        env::set_var("RLOX_RUN_MODE", "P");

        loop {
            print!(">>> ");
            stdout().flush()?;

            let mut line = String::new();
            let len = stdin.read_line(&mut line)?;
            if len == 0 {
                break;
            }
            self.run(line);
            no_error();
        }

        Ok(())
    }

    fn run(&mut self, source: String) {
        let start = SystemTime::now();

        let mut scanner = Scanner::new(source);

        if let Err(err) = scanner.scan_tokens() {
            Self::error(err);
            had_error();
        }

        let mut parser = Parser::new(scanner.tokens);

        match parser.parse() {
            Ok(expression) => match self.interpreter.interpret(expression) {
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
                line,
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
            LoxError::RuntimeError { line, lexeme, msg }
            | LoxError::Break { line, lexeme, msg }
            | LoxError::Continue { line, lexeme, msg } => {
                Self::report(line, format!("at `{}`", lexeme).as_str(), msg.as_str())
            }
            LoxError::IoError { msg } => Self::report(0, "", msg.as_str()),
            LoxError::ParseTokenError { line, msg } => Self::report(line, "", msg),
            LoxError::Return { .. } => (),
        }
    }

    fn report(line: usize, err_pos: &str, msg: &str) {
        if err_pos.is_empty() {
            if line != 0 {
                println!("[line {line}] LoxError: {msg}");
            } else {
                println!("[line] LoxError: {msg}");
            }
        } else if line != 0 {
            println!("[line {line}] LoxError {err_pos}: {msg}");
        } else {
            println!("[line] LoxError {err_pos}: {msg}");
        }
        had_error()
    }
}
