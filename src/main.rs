use std::{
    cell::Cell,
    io::{self, stdin, Write},
    panic,
    process::exit,
};
mod scanner;
use interpreter::Interpreter;
use parser::Parser;
use scanner::Scanner;

mod environment;
mod expr;
mod interpreter;
mod parser;
mod stmt;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn main() -> Result<()> {
    let mut lox = Lox::new();
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.len() {
        0 => lox.run_prompt(),
        1 => lox.run_file(&args[0]),
        _ => {
            println!("Usage: jlox [script]");
            Ok(())
        }
    }
}

struct Lox {
    interpreter: Interpreter,
    had_error: Cell<bool>,
}
impl Lox {
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
            had_error: Cell::new(false),
        }
    }
    fn run_file(&mut self, file: &str) -> Result<()> {
        let code = std::fs::read_to_string(file)?;
        self.run(&code);
        Ok(())
    }

    fn run(&mut self, code: &str) {
        // scanner
        let mut scanner = Scanner::new(code.to_string());
        let tokens = scanner.scan_tokens();

        //        for token in &tokens {
        //            println!("{:?}", token);
        //        }

        self.check_for_error(&scanner);
        if self.had_error.get() {
            exit(65);
        }

        // parser
        let mut parser = Parser::new(tokens);
        let stmts = panic::catch_unwind(move || parser.parse());
        if stmts.is_err() {
            return;
        }

        self.check_for_error(&scanner);
        if self.had_error.get() {
            exit(65);
        }

        self.interpreter.interpret(stmts.unwrap());
        self.check_for_error(&scanner);
        if self.had_error.get() {
            exit(70);
        }
        //println!("{}", AstPrinter {}.print(*expr))
    }
    fn check_for_error(&self, may_contain_error: &dyn LError) {
        self.had_error
            .set(self.had_error.get() & may_contain_error.had_error().get());
    }

    fn run_prompt(&mut self) -> Result<()> {
        let mut line = String::new();
        loop {
            print!("> ");
            io::stdout().flush()?;
            stdin().read_line(&mut line)?;
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
            self.had_error.set(false);
            line.clear();
        }
        Ok(())
    }
}
impl LError for Lox {
    fn had_error(&self) -> &Cell<bool> {
        &self.had_error
    }
}

pub trait LError {
    fn had_error(&self) -> &Cell<bool>;
    fn error(&self, line: usize, message: &str) {
        self.report(line, None, message);
    }

    fn report(&self, line: usize, wheres: Option<String>, message: &str) {
        eprintln!(
            "[line {}] Error{}: {}",
            line,
            wheres.unwrap_or_else(|| "".into()),
            message
        );
        self.had_error().set(true);
    }
}
