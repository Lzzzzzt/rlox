use std::{env, io};

use crate::lib::lox::Lox;

mod lib;

fn main() -> io::Result<()> {
    let mut args = env::args();
    let lox_compiler = Lox::new();
    args.next();

    if args.len() == 0 {
        lox_compiler.run_prompt()?;
    } else if args.len() == 1 {
        lox_compiler.run_file(args.next().unwrap().into())?;
    } else {
        println!("Usage: lox [script]")
    }


    Ok(())
}
