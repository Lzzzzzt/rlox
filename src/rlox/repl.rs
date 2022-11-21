use rustyline::highlight::Highlighter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, Config, Editor, KeyEvent};
use rustyline_derive::{Completer, Helper, Hinter, Validator};

use std::borrow::Cow::{self, Borrowed};

use super::bytecode_interpreter::vm::VirtualMachine;
use super::lox::{self, Lox};
use super::scanner::Scanner;
use super::token::Token;

#[derive(Helper, Completer, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    colored_prompt: String,
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        if default {
            Borrowed(&self.colored_prompt)
        } else {
            Borrowed(prompt)
        }
    }
}

pub struct Repl {
    editor: Editor<MyHelper>,
}

impl Repl {
    pub fn new() -> Self {
        let config = Config::builder()
            .indent_size(8)
            .auto_add_history(true)
            .build();
        let helper = MyHelper {
            colored_prompt: "".to_owned(),
            validator: MatchingBracketValidator::new(),
        };
        let mut editor = Editor::with_config(config).unwrap();

        editor.set_helper(Some(helper));
        editor.bind_sequence(KeyEvent::from('\t'), Cmd::Insert(1, "\t".into()));

        Self { editor }
    }

    pub fn run(&mut self, run_fn: fn(vm: &mut VirtualMachine, tokens: Vec<Token>) -> ()) {
        let mut count = 1;
        let mut vm = VirtualMachine::new();

        loop {
            let p = format!("[{count:4}]: ");
            self.editor.helper_mut().unwrap().colored_prompt = format!("\x1b[1;32m{p}\x1b[0m");
            let readline = self.editor.readline(&p);

            match readline {
                Ok(line) => {
                    let mut scanner = Scanner::new(line);

                    if let Err(err) = scanner.scan_tokens() {
                        Lox::error(err);
                        lox::had_error();
                    }

                    run_fn(&mut vm, scanner.tokens);
                }
                Err(_) => break,
            }
            count += 1;
            lox::no_error();
        }
    }
}
