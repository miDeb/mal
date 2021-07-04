use std::{cell::RefCell, rc::Rc};

use crate::{
    env::Env,
    eval,
    printer::pr_str,
    reader::{read_str, ParseError},
    runtime_errors::RuntimeError,
    value::Value,
};

pub fn init_env(env: &mut Env) {
    env.set(
        "+".to_string(),
        Value::Fn(|list, _| Ok(&list[0] + &list[1])),
    );
    env.set(
        "-".to_string(),
        Value::Fn(|list, _| Ok(&list[0] - &list[1])),
    );
    env.set(
        "*".to_string(),
        Value::Fn(|list, _| Ok(&list[0] * &list[1])),
    );
    env.set("/".to_string(), Value::Fn(|list, _| &list[0] / &list[1]));
    env.set(
        "pr-str".to_string(),
        Value::Fn(|list, _| {
            let mut string = String::new();
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(item, &mut string, true).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "str".to_string(),
        Value::Fn(|list, _| {
            let mut string = String::new();
            for item in list.iter() {
                pr_str(item, &mut string, false).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "prn".to_string(),
        Value::Fn(|list, _| {
            let mut string = String::new();
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(item, &mut string, true).unwrap();
            }
            println!("{}", string);
            Ok(Value::Nil)
        }),
    );
    env.set(
        "println".to_string(),
        Value::Fn(|list, _| {
            let mut string = String::new();
            for (i, item) in list.iter().enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(item, &mut string, false).unwrap();
            }
            println!("{}", string);
            Ok(Value::Nil)
        }),
    );
    env.set(
        "list".to_string(),
        Value::Fn(|list, _| Ok(Value::List(list.to_vec()))),
    );
    env.set(
        "list?".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(matches!(list[0], Value::List(_))))),
    );
    env.set(
        "empty?".to_string(),
        Value::Fn(|list, _| {
            Ok(Value::Bool(
                list[0]
                    .try_as_list_or_vec()
                    .map(|l| l.is_empty())
                    .unwrap_or(true),
            ))
        }),
    );
    env.set(
        "count".to_string(),
        Value::Fn(|list, _| {
            Ok(Value::Number(
                list[0].try_as_list_or_vec().map(|l| l.len()).unwrap_or(0) as i32,
            ))
        }),
    );
    env.set(
        "=".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(list[0] == list[1]))),
    );
    env.set(
        "<".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(list[0] < list[1]))),
    );
    env.set(
        ">".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(list[0] > list[1]))),
    );
    env.set(
        "<=".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(list[0] <= list[1]))),
    );
    env.set(
        ">=".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(list[0] >= list[1]))),
    );

    env.set(
        "read-string".to_string(),
        Value::Fn(|list, _| match read_str(list[0].try_as_str().unwrap()) {
            Ok(v) => Ok(v),
            Err(ParseError::EmptyInput) => Ok(Value::Nil),
            Err(e) => Err(RuntimeError::ParseError(e)),
        }),
    );
    env.set(
        "slurp".to_string(),
        Value::Fn(|list, _| {
            std::fs::read_to_string(list[0].try_as_str().unwrap())
                .map(Value::String)
                .map_err(RuntimeError::IoError)
        }),
    );

    env.set(
        "atom".to_string(),
        Value::Fn(|list, _| Ok(Value::Atom(Rc::new(RefCell::new(list[0].clone()))))),
    );
    env.set(
        "atom?".to_string(),
        Value::Fn(|list, _| Ok(Value::Bool(matches!(list[0], Value::Atom(_))))),
    );
    env.set(
        "deref".to_string(),
        Value::Fn(|list, _| match &list[0] {
            Value::Atom(v) => Ok(v.borrow().clone()),
            v => Err(RuntimeError::NotAnAtom(v.to_string())),
        }),
    );
    env.set(
        "reset!".to_string(),
        Value::Fn(|list, _| match &list[0] {
            Value::Atom(v) => {
                v.replace(list[1].clone());
                Ok(list[1].clone())
            }
            v => Err(RuntimeError::NotAnAtom(v.to_string())),
        }),
    );
    env.set(
        "swap!".to_string(),
        Value::Fn(|list, env| match &list[0] {
            Value::Atom(v) => {
                let mut fn_args = vec![list[1].clone(), v.borrow().clone()];
                fn_args.extend(list[2..].iter().cloned());
                let invocation = Value::List(fn_args);
                let result = eval(invocation, env)?;
                v.replace(result.clone());
                Ok(result)
            }
            v => Err(RuntimeError::NotAnAtom(v.to_string())),
        }),
    );
}
