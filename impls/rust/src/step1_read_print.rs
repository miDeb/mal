use std::io::Write;

use reader::{ParseError, ParseResult};
use rustyline::Editor;
use value::Value;

mod printer;
mod reader;
mod runtime_errors;
mod tokenize;
mod value;

fn main() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    while let Ok(line) = rl.readline("user> ") {
        rl.add_history_entry(&line);
        match read(&line) {
            Ok(value) => {
                println!("{}", eval(value));
            }
            Err(ParseError::EmptyInput) => {}
            Err(e) => {
                eprintln!("{}", e);
            }
        };
        std::io::stdout().flush().unwrap();
    }
    rl.save_history("history.txt").unwrap();
}

fn read(input: &str) -> ParseResult<Value> {
    reader::read_str(input)
}

fn eval(input: Value) -> Value {
    input
}