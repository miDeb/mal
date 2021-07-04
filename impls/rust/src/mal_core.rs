use std::{cell::RefCell, convert::TryInto, rc::Rc};

use crate::{
    env::Env,
    eval,
    printer::pr_str,
    reader::{read_str, ParseError},
    runtime_errors::{self, error_to_string_with_ctx, RuntimeResult},
    value::{MalFunction, Value},
};

pub fn init_env(env: &mut Env) {
    fn make_fn_val(f: fn(&[Value], Rc<RefCell<Env>>) -> RuntimeResult<Value>) -> Value {
        Value::Fn(MalFunction(f))
    }
    env.set(
        "+".to_string(),
        make_fn_val(|list, _| Ok(&list[0] + &list[1])),
    );
    env.set(
        "-".to_string(),
        make_fn_val(|list, _| Ok(&list[0] - &list[1])),
    );
    env.set(
        "*".to_string(),
        make_fn_val(|list, _| Ok(&list[0] * &list[1])),
    );
    env.set("/".to_string(), make_fn_val(|list, _| &list[0] / &list[1]));
    env.set(
        "pr-str".to_string(),
        make_fn_val(|list, _| {
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
        make_fn_val(|list, _| {
            let mut string = String::new();
            for item in list.iter() {
                pr_str(item, &mut string, false).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "prn".to_string(),
        make_fn_val(|list, _| {
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
        make_fn_val(|list, _| {
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
        make_fn_val(|list, _| Ok(Value::List(list.to_vec()))),
    );
    env.set(
        "list?".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(matches!(list[0], Value::List(_))))),
    );
    env.set(
        "empty?".to_string(),
        make_fn_val(|list, _| {
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
        make_fn_val(|list, _| {
            Ok(Value::Number(
                list[0].try_as_list_or_vec().map(|l| l.len()).unwrap_or(0) as i32,
            ))
        }),
    );
    env.set(
        "=".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(list[0] == list[1]))),
    );
    env.set(
        "<".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(list[0] < list[1]))),
    );
    env.set(
        ">".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(list[0] > list[1]))),
    );
    env.set(
        "<=".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(list[0] <= list[1]))),
    );
    env.set(
        ">=".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(list[0] >= list[1]))),
    );

    env.set(
        "read-string".to_string(),
        make_fn_val(|list, _| match read_str(list[0].try_as_str()?) {
            Ok(v) => Ok(v),
            Err(ParseError::EmptyInput) => Ok(Value::Nil),
            Err(e) => Err(error_to_string_with_ctx("parsing failed", e)),
        }),
    );
    env.set(
        "slurp".to_string(),
        make_fn_val(|list, _| {
            let file = list[0].try_as_str()?;
            std::fs::read_to_string(file)
                .map(Value::String)
                .map_err(|e| {
                    runtime_errors::error_to_string_with_ctx(
                        format!("failed to read file {}", file),
                        e,
                    )
                })
        }),
    );

    env.set(
        "atom".to_string(),
        make_fn_val(|list, _| Ok(Value::Atom(Rc::new(RefCell::new(list[0].clone()))))),
    );
    env.set(
        "atom?".to_string(),
        make_fn_val(|list, _| Ok(Value::Bool(matches!(list[0], Value::Atom(_))))),
    );
    env.set(
        "deref".to_string(),
        make_fn_val(|list, _| match &list[0] {
            Value::Atom(v) => Ok(v.borrow().clone()),
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );
    env.set(
        "reset!".to_string(),
        make_fn_val(|list, _| match &list[0] {
            Value::Atom(v) => {
                v.replace(list[1].clone());
                Ok(list[1].clone())
            }
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );
    env.set(
        "swap!".to_string(),
        make_fn_val(|list, env| match &list[0] {
            Value::Atom(v) => {
                let mut fn_args = vec![list[1].clone(), v.borrow().clone()];
                fn_args.extend(list[2..].iter().cloned());
                let invocation = Value::List(fn_args);
                let result = eval(invocation, env)?;
                v.replace(result.clone());
                Ok(result)
            }
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );

    env.set(
        "cons".to_string(),
        make_fn_val(|args, _| {
            let mut list = args[1].clone().try_into_list_or_vec().unwrap();
            list.insert(0, args[0].clone());
            Ok(Value::List(list))
        }),
    );
    env.set(
        "concat".to_string(),
        make_fn_val(|args, _| {
            let mut list = Vec::new();
            for arg in args {
                list.append(&mut arg.clone().try_into_list_or_vec().unwrap());
            }
            Ok(Value::List(list))
        }),
    );

    env.set(
        "vec".to_string(),
        make_fn_val(|args, _| Ok(Value::Vec(args[0].clone().try_into_list_or_vec().unwrap()))),
    );

    env.set(
        "nth".to_string(),
        make_fn_val(|args, _| match &args[0] {
            Value::List(l) | Value::Vec(l) if !l.is_empty() => {
                let index_unconverted: i32 = args[1].try_as_number().unwrap();
                let index: usize = index_unconverted
                    .try_into()
                    .map_err(|_| runtime_errors::out_of_bounds(l.len(), index_unconverted))?;

                Ok(l.get(index)
                    .ok_or_else(|| runtime_errors::out_of_bounds(l.len(), index_unconverted))?
                    .clone())
            }
            Value::List(_) | Value::Vec(_) | Value::Nil => Ok(Value::Nil),
            v => Err(runtime_errors::not_a("list", v)),
        }),
    );
    env.set(
        "first".to_string(),
        make_fn_val(|args, _| match &args[0] {
            Value::List(l) | Value::Vec(l) if !l.is_empty() => Ok(l[0].clone()),
            Value::List(_) | Value::Vec(_) | Value::Nil => Ok(Value::Nil),
            v => Err(runtime_errors::not_a("list", v)),
        }),
    );
    env.set(
        "rest".to_string(),
        make_fn_val(|args, _| match &args[0] {
            Value::List(l) | Value::Vec(l) if !l.is_empty() => Ok(Value::List(l[1..].to_vec())),
            Value::List(_) | Value::Vec(_) | Value::Nil => Ok(Value::List(Vec::new())),
            v => Err(runtime_errors::not_a("list", v)),
        }),
    );
}
