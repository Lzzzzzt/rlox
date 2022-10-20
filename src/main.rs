mod lib;

use std::{env};
use crate::lib::lox::Lox;


fn main() {
    let mut args = env::args();
    let lox_compiler = Lox::new();
    args.next();


    if args.len() == 0 {
        lox_compiler.run_prompt().unwrap();
    } else if args.len() == 1 {
        lox_compiler.run_file(args.next().unwrap().into()).unwrap();
    } else {
        println!("Usage: lox [script]")
    }
}
