use std::fs::read_to_string;
use std::io::{Result, stdout, Write};
use std::path::PathBuf;

use crate::lib::scanner::Scanner;

static mut HAD_ERROR: bool = false;

fn is_error() -> bool {
    unsafe {
        HAD_ERROR
    }
}

fn no_error() {
    unsafe {
        HAD_ERROR = false
    }
}

fn had_error() {
    unsafe {
        HAD_ERROR = true
    }
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
        };

        Ok(())
    }

    fn run(&self, source: String) -> Result<()> {
        let mut scanner = Scanner::new(source);
        scanner.scan_tokens();

        scanner.tokens.iter().for_each(|token| {
            println!("{}", token)
        });

        Ok(())
    }

    pub fn error(line: usize, msg: &str) {
        Self::report(line, "", msg);
    }

    fn report(line: usize, err_pos: &str, msg: &str) {
        println!("[line {line}] Error {err_pos}: {msg}");
        had_error()
    }
}