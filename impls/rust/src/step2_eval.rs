#![feature(bindings_after_at)]

use std::{collections::HashMap, io::Write};

use env::Env;
use reader::{ParseError, ParseResult};
use runtime_errors::{RuntimeError, RuntimeResult};
use rustyline::Editor;
use value::Value;

mod env;
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
                print(eval(value, &Env::new()));
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

fn eval(input: Value, env: &Env) -> RuntimeResult<Value> {
    match input {
        Value::List(l) if l.is_empty() => Ok(Value::List(l)),
        Value::List(l) => {
            let new_list = eval_ast(Value::List(l), env)?.into_list();
            let (first, rest) = new_list.split_first().unwrap();
            let f = first.as_fn()?;
            f(rest)
        }
        v => eval_ast(v, env),
    }
}

fn print(value: RuntimeResult<Value>) {
    match value {
        Ok(value) => {
            println!("{}", value);
        }
        Err(e) => eprintln!("{}", e),
    }
}

fn eval_ast(value: Value, env: &Env) -> RuntimeResult<Value> {
    match value {
        Value::List(list) => {
            let mut new_list = Vec::with_capacity(list.len());
            for v in list {
                new_list.push(eval(v, env)?);
            }
            Ok(Value::List(new_list))
        }
        Value::Vec(vec) => {
            let mut new_vec = Vec::with_capacity(vec.len());
            for v in vec {
                new_vec.push(eval(v, env)?);
            }
            Ok(Value::Vec(new_vec))
        }
        Value::Map(map) => {
            let mut new_map = HashMap::with_capacity(map.len());
            for (k, v) in map {
                new_map.insert(k, eval(v, env)?);
            }
            Ok(Value::Map(new_map))
        }
        Value::Symbol(s) => env
            .map
            .get(&s)
            .cloned()
            .ok_or(RuntimeError::FnNotFound(s)),
        v => Ok(v),
    }
}
