mod rlox;

use crate::rlox::lox::Lox;

use std::env;

fn main() {
    let mut args = env::args();
    let rlox = Lox::new();
    args.next();

    if args.len() == 0 {
        rlox.run_prompt().unwrap();
    } else if args.len() == 1 {
        rlox.run_file(args.next().unwrap().into()).unwrap();
    } else {
        println!("Usage: rlox [script]")
    }
}
