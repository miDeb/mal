#![feature(bindings_after_at)]

use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

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
    let env = Rc::new(RefCell::new(Env::new(None)));
    while let Ok(line) = rl.readline("user> ") {
        rl.add_history_entry(&line);
        match read(&line) {
            Ok(value) => {
                let mut borrowed_env = env.borrow_mut();
                borrowed_env.set(
                    "+".to_string(),
                    Value::Fn(Rc::new(|list| Ok(&list[0] + &list[1]))),
                );
                borrowed_env.set(
                    "-".to_string(),
                    Value::Fn(Rc::new(|list| Ok(&list[0] - &list[1]))),
                );
                borrowed_env.set(
                    "*".to_string(),
                    Value::Fn(Rc::new(|list| Ok(&list[0] * &list[1]))),
                );
                borrowed_env.set(
                    "/".to_string(),
                    Value::Fn(Rc::new(|list| &list[0] / &list[1])),
                );
                drop(borrowed_env);
                print(eval(value, &env));
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

fn eval(input: Value, env: &Rc<RefCell<Env>>) -> RuntimeResult<Value> {
    match input {
        Value::List(l) if l.is_empty() => Ok(Value::List(l)),
        Value::List(l) if matches!(&l[0], Value::Symbol(n) if n == "def!") => {
            // TODO: arity checking
            let mut iter = l.into_iter();
            iter.next();
            let key = iter.next().unwrap();
            let val = eval(iter.next().unwrap(), env)?;
            env.borrow_mut().set(
                key.into_env_map_key()
                    .map_err(|e| RuntimeError::InvalidMapKey(e.to_string()))?,
                val.clone(),
            );
            Ok(val)
        }
        Value::List(l) if matches!(&l[0], Value::Symbol(n) if n == "let*") => {
            let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
            let mut args_iter = l.into_iter();
            args_iter.next();
            let mut bindings_iter = args_iter
                .next()
                .unwrap()
                .try_into_list_or_vec()
                .unwrap()
                .into_iter();
            while let Some(key) = bindings_iter.next() {
                let value = eval(bindings_iter.next().unwrap(), &new_env)?;
                new_env.borrow_mut().set(
                    key.into_env_map_key()
                        .map_err(|e| RuntimeError::InvalidMapKey(e.to_string()))?,
                    value,
                );
            }

            eval(args_iter.next().unwrap(), &new_env)
        }
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

fn eval_ast(value: Value, env: &Rc<RefCell<Env>>) -> RuntimeResult<Value> {
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
        Value::Symbol(s) => Env::get(env, &s),
        v => Ok(v),
    }
}
