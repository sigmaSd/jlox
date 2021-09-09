use jlox::{Lox, Result};

fn main() -> Result<()> {
    let mut lox = Lox::default();
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
