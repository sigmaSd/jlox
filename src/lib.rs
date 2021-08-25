use std::{
    io::{self, Write},
    path::Path,
};

mod ast;
mod expr;
mod interpreter;
mod parser;
mod scanner;
mod stmt;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct Lox {
    interpreter: Interpreter,
}
impl Default for Lox {
    fn default() -> Self {
        std::panic::set_hook(Box::new(|info| {
            if info.payload().downcast_ref::<&str>() == Some(&"<Throw>") {
                // Don't show any message on Throw errors
            } else {
                eprintln!("{}", info);
            }
        }));
        Lox {
            interpreter: Interpreter::default(),
        }
    }
}

impl Lox {
    pub fn run(&mut self, code: &str) {
        // scanner
        let mut scanner = Scanner::new(code.to_string());
        let tokens = scanner.scan_tokens();

        // parser
        let mut parser = Parser::new(tokens);
        let stmts = std::panic::catch_unwind(move || parser.parse());
        if stmts.is_err() {
            return;
        }
        let stmts = stmts.unwrap();

        // interpreter
        // Note: Catch is not very robust, because the fields of the interpreter are still shared
        let mut interpreter = self.interpreter.clone();
        let interpret_result = std::panic::catch_unwind(move || {
            interpreter.interpret(stmts);
            interpreter
        });
        if let Ok(interpreter) = interpret_result {
            self.interpreter = interpreter;
        }
    }

    pub fn run_file<P: AsRef<Path>>(&mut self, file: P) -> Result<()> {
        let code = std::fs::read_to_string(file)?;
        self.run(&code);
        Ok(())
    }

    pub fn run_prompt(&mut self) -> Result<()> {
        let mut line = String::new();
        loop {
            print!("> ");
            io::stdout().flush()?;
            io::stdin().read_line(&mut line)?;
            let code = line.trim_end(); // remove \n
            if code.is_empty() {
                break;
            }
            // always print in a repl
            let repl_it = |code: &str| {
                if !code.starts_with("fun ") && !code.ends_with(';') {
                    format!("print {};", code)
                } else {
                    code.to_string()
                }
            };
            self.run(&repl_it(code));
            line.clear();
        }
        Ok(())
    }
}
