use std::fs::read_to_string;
use std::io::{Result, stdout, Write};
use std::path::PathBuf;

use crate::lib::parser::Parser;
use crate::lib::scanner::Scanner;
use crate::lib::token::Token;
use crate::lib::token_type::TokenType;

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

pub struct Lox {}

impl Lox {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_file(self, path: PathBuf) -> Result<()> {
        let string = read_to_string(path)?;
        self.run(string)?;

        if is_error() {
            panic!("Exit because error before!");
        }

        Ok(())
    }

    pub fn run_prompt(self) -> Result<()> {
        let stdin = std::io::stdin();

        loop {
            print!("> ");
            stdout().flush()?;

            let mut line = String::new();
            let len = stdin.read_line(&mut line)?;
            if len == 0 {
                break;
            }
            self.run(line)?;
            no_error();
        }

        Ok(())
    }

    fn run(&self, source: String) -> Result<()> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        // println!("{:#?}", scanner.tokens);

        let mut parser = Parser::new(scanner.tokens);

        if let Ok(expression) = parser.parse() {
            println!("{:#?}", expression);
        };

        Ok(())
    }

    pub fn error(line: usize, msg: &str) {
        Self::report(line, "", msg);
    }

    pub fn token_error(token: &Token, msg: &str) {
        if token.token_type == TokenType::Eof {
            Self::report(token.line, "at end", msg)
        } else {
            Self::report(token.line, format!("at `{}`", token.lexeme).as_str(), msg)
        }
    }

    fn report(line: usize, err_pos: &str, msg: &str) {
        if err_pos.is_empty() {
            println!("[line {line}] LoxError: {msg}");
        } else {
            println!("[line {line}] LoxError({err_pos}): {msg}");
        }
        had_error()
    }
}
