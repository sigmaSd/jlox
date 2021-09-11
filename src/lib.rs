use std::{
    io::{self, Write},
    path::Path,
    process,
};

mod ast;
mod expr;
mod interpreter;
mod parser;
mod resolver;
mod scanner;
mod stmt;
use interpreter::Interpreter;
use parser::Parser;
use resolver::Resolver;
use scanner::Scanner;
use trycatch::{catch, CatchError, ExceptionDowncast};

use crate::interpreter::RuntimeError;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default)]
pub struct Lox {
    interpreter: Interpreter,
}

impl Lox {
    pub fn run(&mut self, code: &str) {
        // scanner
        let mut scanner = Scanner::new(code.to_string());
        let tokens = scanner.scan_tokens();
        if scanner.had_error {
            process::exit(65)
        }

        // parser
        let mut parser = Parser::new(tokens);
        let stmts = parser.parse();
        if parser.had_error {
            process::exit(65)
        }

        // resolver
        let mut resolver = Resolver::new(self.interpreter.clone());
        resolver.resolve_stmts(&stmts);

        if resolver.had_error {
            process::exit(65)
        }

        // interpreter
        let mut interpreter = self.interpreter.clone();
        let interpret_result = catch(move || {
            interpreter.interpret(stmts);
            interpreter
        });
        match interpret_result {
            Ok(interpreter) => self.interpreter = interpreter,
            Err(CatchError::Exception(e)) => {
                let runtime_error: RuntimeError = e.downcast();
                eprintln!("{}", runtime_error.to_string());
                process::exit(70);
            }
            Err(e) => panic!("{:?}", e),
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
