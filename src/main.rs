mod rlox;

use crate::rlox::lox::Lox;

use std::env;

fn main() {
    let mut args = env::args();
    args.next();

    if args.len() == 0 {
        Lox::run_prompt().unwrap();
    } else if args.len() == 1 {
        Lox::run_file(args.next().unwrap().into()).unwrap();
    } else {
        println!("Usage: rlox [script]")
    }
}
