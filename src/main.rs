mod ast;
mod expr;
mod interpreter;
mod lox;
mod parser;
mod scanner;
mod stmt;
use lox::Lox;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
