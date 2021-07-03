#![feature(bindings_after_at)]

use mal_core::init_env;
use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use env::Env;
use reader::{ParseError, ParseResult};
use runtime_errors::{RuntimeError, RuntimeResult};
use rustyline::Editor;
use value::Value;

use crate::value::Closure;

mod env;
mod mal_core;
mod printer;
mod reader;
mod runtime_errors;
mod tokenize;
mod value;

fn main() {
    let mut rl = Editor::<()>::new();
    rl.load_history("history.txt").ok();
    let env = Rc::new(RefCell::new(Env::new(None)));
    eval(
        read("(def! not (fn* (a) (if a false true)))").unwrap(),
        env.clone(),
    )
    .unwrap();
    while let Ok(line) = rl.readline("user> ") {
        rl.add_history_entry(&line);
        match read(&line) {
            Ok(value) => {
                let mut borrowed_env = env.borrow_mut();
                init_env(&mut borrowed_env);
                drop(borrowed_env);
                print(eval(value, env.clone()));
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

fn eval(mut input: Value, mut env: Rc<RefCell<Env>>) -> RuntimeResult<Value> {
    loop {
        break match input {
            Value::List(l) if l.is_empty() => Ok(Value::List(l)),
            Value::List(l) if matches!(&l[0], Value::Symbol(n) if n == "def!") => {
                // TODO: arity checking
                let mut iter = l.into_iter();
                iter.next();
                let key = iter.next().unwrap();
                let val = eval(iter.next().unwrap(), env.clone())?;
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
                    let value = eval(bindings_iter.next().unwrap(), new_env.clone())?;
                    new_env.borrow_mut().set(
                        key.into_env_map_key()
                            .map_err(|e| RuntimeError::InvalidMapKey(e.to_string()))?,
                        value,
                    );
                }

                input = args_iter.next().unwrap();
                env = new_env;
                continue;
            }
            Value::List(l) if matches!(&l[0], Value::Symbol(n) if n == "do") => {
                // Change from the "official" algorithm, but is semantically the same.
                let len = l.len() - 1;
                let mut iter = l.into_iter();
                iter.next();
                for _ in 0..(len - 1) {
                    eval(iter.next().unwrap(), env.clone())?;
                }
                input = iter.next().unwrap();
                continue;
            }
            Value::List(l) if matches!(&l[0], Value::Symbol(n) if n == "if") => {
                let mut iter = l.into_iter();
                iter.next();
                let cond = eval(iter.next().unwrap(), env.clone())?;
                let then = iter.next().unwrap();
                if !matches!(cond, Value::Bool(false) | Value::Nil) {
                    input = then;
                    continue;
                } else if let Some(else_case) = iter.next() {
                    input = else_case;
                    continue;
                } else {
                    Ok(Value::Nil)
                }
            }
            Value::List(l) if matches!(&l[0], Value::Symbol(n) if n == "fn*") => {
                let mut iter = l.into_iter();
                iter.next();
                let env = env;
                let params = iter.next().unwrap().try_into_list_or_vec().unwrap();
                let ast = iter.next().unwrap();
                Ok(Value::Closure(Rc::new(Closure { ast, params, env })))
            }
            Value::List(l) => {
                let new_list = eval_ast(Value::List(l), env)?.into_list();
                let mut args = new_list.into_iter();
                let first = args.next().unwrap();
                match first {
                    Value::Fn(f) => f(args.as_slice()),
                    Value::Closure(closure) => {
                        input = closure.ast.clone();
                        env = Rc::new(RefCell::new(Env::new_with_binds(
                            Some(closure.env.clone()),
                            closure.params.clone().into_iter(),
                            args,
                        )?));
                        continue;
                    }
                    no_fun => return Err(RuntimeError::NotAFunction(no_fun.to_string())),
                }
            }
            v => eval_ast(v, env),
        };
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

fn eval_ast(value: Value, env: Rc<RefCell<Env>>) -> RuntimeResult<Value> {
    // TODO: don't allocate here. We can reuse Vecs and Maps.
    match value {
        Value::List(list) => {
            let mut new_list = Vec::with_capacity(list.len());
            for v in list {
                new_list.push(eval(v, env.clone())?);
            }
            Ok(Value::List(new_list))
        }
        Value::Vec(vec) => {
            let mut new_vec = Vec::with_capacity(vec.len());
            for v in vec {
                new_vec.push(eval(v, env.clone())?);
            }
            Ok(Value::Vec(new_vec))
        }
        Value::Map(map) => {
            let mut new_map = HashMap::with_capacity(map.len());
            for (k, v) in map {
                new_map.insert(k, eval(v, env.clone())?);
            }
            Ok(Value::Map(new_map))
        }
        Value::Symbol(s) => Env::get(&env, &s),
        v => Ok(v),
    }
}
