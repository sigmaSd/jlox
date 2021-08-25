use std::{
    io::{self, Write},
    panic,
};

use crate::{interpreter::Interpreter, parser::Parser, scanner::Scanner, Result};

pub struct Lox {
    interpreter: Interpreter,
}
impl Lox {
    pub fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }
    pub fn run_file(&mut self, file: &str) -> Result<()> {
        let code = std::fs::read_to_string(file)?;
        self.run(&code);
        Ok(())
    }

    pub fn run(&mut self, code: &str) {
        // scanner
        let mut scanner = Scanner::new(code.to_string());
        let tokens = scanner.scan_tokens();

        //        for token in &tokens {
        //            println!("{:?}", token);
        //        }

        // parser
        let mut parser = Parser::new(tokens);
        let stmts = panic::catch_unwind(move || parser.parse());
        if stmts.is_err() {
            return;
        }

        self.interpreter.interpret(stmts.unwrap());
        //println!("{}", AstPrinter {}.print(*expr))
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
                if !code.ends_with(';') {
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
