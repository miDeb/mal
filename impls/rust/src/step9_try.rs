#![feature(bindings_after_at)]

use mal_core::init_env;
use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc};

use env::Env;
use reader::{ParseError, ParseResult};
use runtime_errors::RuntimeResult;
use rustyline::Editor;
use value::{HostFn, Value};

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
    eval(
        read(r#"(def! load-file (fn* (f) (eval (read-string (str "(do " (slurp f) "\nnil)")))))"#)
            .unwrap(),
        env.clone(),
    )
    .unwrap();
    eval(
        read(r#"(defmacro! cond (fn* (& xs) (if (> (count xs) 0) (list 'if (first xs) (if (> (count xs) 1) (nth xs 1) (throw "odd number of forms to cond")) (cons 'cond (rest (rest xs)))))))"#)
            .unwrap(),
        env.clone(),
    )
    .unwrap();
    env.borrow_mut()
        .set("eval".to_string(), Value::HostFn(HostFn::Eval(env.clone())));

    if let Some(file_name) = std::env::args().nth(1) {
        let argv = Value::List(std::env::args().skip(2).map(Value::String).collect());
        env.borrow_mut().set("*ARGV*", argv);
        match re(
            format!(r#"(load-file "{}")"#, file_name.replace('"', r#"\""#)),
            &env,
        ) {
            Some(r) if r.is_err() => print(r),
            // don't print the nil from reading the file.
            _ => {}
        }
        return;
    }
    // we can't have any args if we reach this point, but *ARGV* must be present.
    env.borrow_mut().set("*ARGV*", Value::List(vec![]));

    while let Ok(line) = rl.readline("user> ") {
        rl.add_history_entry(&line);
        if let Some(a) = re(line, &env) {
            print(a)
        }
        std::io::stdout().flush().unwrap();
    }
    rl.save_history("history.txt").unwrap();
}

fn re(line: String, env: &Rc<RefCell<Env>>) -> Option<RuntimeResult<Value>> {
    match read(&line) {
        Ok(value) => {
            let mut borrowed_env = env.borrow_mut();
            init_env(&mut borrowed_env);
            drop(borrowed_env);
            return Some(eval(value, env.clone()));
        }
        Err(ParseError::EmptyInput) => {}
        Err(e) => {
            eprintln!("{}", e);
        }
    };
    None
}

fn read(input: &str) -> ParseResult<Value> {
    reader::read_str(input)
}

fn eval(mut input: Value, mut env: Rc<RefCell<Env>>) -> RuntimeResult<Value> {
    loop {
        input = macro_expand(input, &env)?;
        break match input {
            Value::List(l) if l.is_empty() => Ok(Value::List(l)),
            Value::List(l) => {
                match &l[0] {
                    Value::Symbol(n) if n == "def!" => {
                        // TODO: arity checking
                        let mut iter = l.into_iter();
                        let key = iter.nth(1).unwrap();
                        let val = eval(iter.next().unwrap(), env.clone())?;
                        env.borrow_mut()
                            .set(key.try_into_env_map_key()?, val.clone());
                        Ok(val)
                    }
                    Value::Symbol(n) if n == "let*" => {
                        let new_env = Rc::new(RefCell::new(Env::new(Some(env.clone()))));
                        let mut args_iter = l.into_iter();
                        args_iter.next();
                        let mut bindings_iter = args_iter
                            .next()
                            .unwrap()
                            .try_into_list_or_vec()?
                            .into_iter();
                        while let Some(key) = bindings_iter.next() {
                            let value = eval(bindings_iter.next().unwrap(), new_env.clone())?;
                            new_env.borrow_mut().set(key.try_into_env_map_key()?, value);
                        }

                        input = args_iter.next().unwrap();
                        env = new_env;
                        continue;
                    }
                    Value::Symbol(n) if n == "do" => {
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
                    Value::Symbol(n) if n == "if" => {
                        let mut iter = l.into_iter();
                        let cond = eval(iter.nth(1).unwrap(), env.clone())?;
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
                    Value::Symbol(n) if n == "fn*" => {
                        let mut iter = l.into_iter();
                        let env = env;
                        let params = iter.nth(1).unwrap().try_into_list_or_vec()?;
                        let ast = iter.next().unwrap();
                        Ok(Value::Closure(Rc::new(Closure {
                            ast,
                            params,
                            env,
                            is_macro: false,
                        })))
                    }
                    Value::Symbol(n) if n == "defmacro!" => {
                        let mut iter = l.into_iter();
                        let key = iter.nth(1).unwrap();
                        let val = match eval(iter.next().unwrap(), env.clone())? {
                            Value::Closure(c) => {
                                let mut closure = c.as_ref().clone();
                                closure.is_macro = true;
                                Value::Closure(Rc::new(closure))
                            }
                            _ => {
                                // TODO: error
                                todo!()
                            }
                        };
                        env.borrow_mut()
                            .set(key.try_into_env_map_key()?, val.clone());
                        Ok(val)
                    }
                    Value::Symbol(n) if n == "quote" => Ok(l.into_iter().nth(1).unwrap()),
                    Value::Symbol(n) if n == "quasiquoteexpand" => {
                        let ast = l.into_iter().nth(1).unwrap();
                        quasiquote(ast)
                    }
                    Value::Symbol(n) if n == "quasiquote" => {
                        let ast = l.into_iter().nth(1).unwrap();
                        input = quasiquote(ast)?;
                        continue;
                    }
                    Value::Symbol(n) if n == "macroexpand" => {
                        let ast = l.into_iter().nth(1).unwrap();
                        macro_expand(ast, &env)
                    }
                    Value::Symbol(n) if n == "try*" => {
                        let mut args = l.into_iter().skip(1);
                        let result = eval(args.next().unwrap(), env.clone());
                        match result {
                            Ok(value) => return Ok(value),
                            Err(err) => {
                                if let Some(catch_block) = args.next() {
                                    let mut catch_block =
                                        catch_block.into_list().into_iter().skip(1);
                                    let bind_error_to = catch_block.next().unwrap();
                                    let to_eval = catch_block.next().unwrap();
                                    input = to_eval;
                                    env = Rc::new(RefCell::new(Env::new_with_binds(
                                        Some(env),
                                        std::iter::once(bind_error_to),
                                        std::iter::once(err),
                                    )?));
                                    continue;
                                } else {
                                    return Err(err);
                                }
                            }
                        }
                    }
                    _ => {
                        let mut new_list = eval_ast(Value::List(l), env.clone())?.into_list();
                        while matches!(&new_list[0], Value::HostFn(HostFn::Apply)) {
                            new_list.remove(0);
                            if new_list.len() > 1 {
                                match new_list.pop().unwrap() {
                                    Value::Vec(l) | Value::List(l) => {
                                        new_list.extend(l.into_iter());
                                    }
                                    not_a_list => new_list.push(not_a_list),
                                }
                            }
                        }
                        let mut args = new_list.into_iter();
                        let first = args.next().unwrap();
                        match first {
                            Value::HostFn(HostFn::Apply) => unreachable!(),
                            Value::HostFn(HostFn::Eval(eval_env)) => {
                                env = eval_env;
                                input = args.next().unwrap();
                                continue;
                            }
                            Value::HostFn(HostFn::ByPtr(f)) => (f.0)(args.as_slice(), env),
                            Value::Closure(closure) => {
                                input = closure.ast.clone();
                                env = Rc::new(RefCell::new(Env::new_with_binds(
                                    Some(closure.env.clone()),
                                    closure.params.clone().into_iter(),
                                    args,
                                )?));
                                continue;
                            }
                            no_fun => return Err(runtime_errors::not_a("function", &no_fun)),
                        }
                    }
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
        Err(e) => eprintln!("ERROR: {}", e),
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

fn quasiquote(ast: Value) -> RuntimeResult<Value> {
    match ast {
        Value::List(l) if matches!(l.get(0), Some(Value::Symbol(n)) if n == "unquote") => {
            Ok(l.into_iter().nth(1).unwrap())
        }
        Value::List(l) => process_list(l),
        Value::Vec(ast) => Ok(Value::List(vec![
            Value::Symbol("vec".to_string()),
            process_list(ast)?,
        ])),
        v @ Value::Map(_) | v @ Value::Symbol(_) => {
            Ok(Value::List(vec![Value::Symbol("quote".to_string()), v]))
        }
        v => Ok(v),
    }
}

fn process_list(list: Vec<Value>) -> RuntimeResult<Value> {
    let mut result = Vec::new();
    for elt in list.into_iter().rev() {
        result = match elt {
            Value::List(l) if matches!(l.get(0), Some(Value::Symbol(n)) if n == "splice-unquote") =>
            {
                vec![
                    Value::Symbol("concat".to_string()),
                    l.into_iter().nth(1).unwrap(),
                    Value::List(result),
                ]
            }
            v => {
                vec![
                    Value::Symbol("cons".to_string()),
                    quasiquote(v)?,
                    Value::List(result),
                ]
            }
        }
    }
    Ok(Value::List(result))
}

fn as_macro_call(ast: Value, env: &Rc<RefCell<Env>>) -> Option<Rc<Closure>> {
    match ast {
        Value::List(v) if !v.is_empty() => {
            if let Ok(k) = v.into_iter().next().unwrap().try_into_env_map_key() {
                match Env::get(env, &k) {
                    Ok(Value::Closure(c)) if c.is_macro => Some(c),
                    _ => None,
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

fn macro_expand(mut ast: Value, env: &Rc<RefCell<Env>>) -> RuntimeResult<Value> {
    while let Some(closure) = as_macro_call(ast.clone(), env) {
        ast = eval(
            closure.ast.clone(),
            Rc::new(RefCell::new(Env::new_with_binds(
                Some(closure.env.clone()),
                closure.params.clone().into_iter(),
                ast.into_list().into_iter().skip(1),
            )?)),
        )?;
    }
    Ok(ast)
}
