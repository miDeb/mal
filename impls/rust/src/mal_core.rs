use std::{cell::RefCell, collections::HashMap, convert::TryInto, rc::Rc};

use crate::{
    env::Env,
    eval_fn_no_tco,
    printer::pr_str,
    reader::{read_str, ParseError},
    runtime_errors::{self, error_to_string_with_ctx, RuntimeResult},
    value::{HostFn, MalFnPtr, Value},
};

pub fn init_env(env: &mut Env) {
    fn make_fn_val(f: fn(&[Value], Rc<RefCell<Env>>) -> RuntimeResult<Value>) -> Value {
        Value::HostFn(HostFn::ByPtr(MalFnPtr(f)), Box::new(Value::Nil))
    }
    env.set("+", make_fn_val(|list, _| Ok(&list[0] + &list[1])));
    env.set("-", make_fn_val(|list, _| Ok(&list[0] - &list[1])));
    env.set("*", make_fn_val(|list, _| Ok(&list[0] * &list[1])));
    env.set("/", make_fn_val(|list, _| &list[0] / &list[1]));
    env.set(
        "pr-str",
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
        "str",
        make_fn_val(|list, _| {
            let mut string = String::new();
            for item in list.iter() {
                pr_str(item, &mut string, false).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "prn",
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
        "println",
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
        "list",
        make_fn_val(|list, _| Ok(Value::List(list.to_vec(), Box::new(Value::Nil)))),
    );
    env.set(
        "list?",
        make_fn_val(|list, _| Ok(Value::Bool(matches!(list[0], Value::List(_, _))))),
    );
    env.set(
        "empty?",
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
        "count",
        make_fn_val(|list, _| {
            Ok(Value::Number(
                list[0].try_as_list_or_vec().map(|l| l.len()).unwrap_or(0) as i32,
            ))
        }),
    );
    env.set(
        "=",
        make_fn_val(|list, _| Ok(Value::Bool(list[0] == list[1]))),
    );
    env.set(
        "<",
        make_fn_val(|list, _| Ok(Value::Bool(list[0] < list[1]))),
    );
    env.set(
        ">",
        make_fn_val(|list, _| Ok(Value::Bool(list[0] > list[1]))),
    );
    env.set(
        "<=",
        make_fn_val(|list, _| Ok(Value::Bool(list[0] <= list[1]))),
    );
    env.set(
        ">=",
        make_fn_val(|list, _| Ok(Value::Bool(list[0] >= list[1]))),
    );

    env.set(
        "read-string",
        make_fn_val(|list, _| match read_str(list[0].try_as_str()?) {
            Ok(v) => Ok(v),
            Err(ParseError::EmptyInput) => Ok(Value::Nil),
            Err(e) => Err(error_to_string_with_ctx("parsing failed", e)),
        }),
    );
    env.set(
        "slurp",
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
        "atom",
        make_fn_val(|list, _| Ok(Value::Atom(Rc::new(RefCell::new(list[0].clone()))))),
    );
    env.set(
        "atom?",
        make_fn_val(|list, _| Ok(Value::Bool(matches!(list[0], Value::Atom(_))))),
    );
    env.set(
        "deref",
        make_fn_val(|list, _| match &list[0] {
            Value::Atom(v) => Ok(v.borrow().clone()),
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );
    env.set(
        "reset!",
        make_fn_val(|list, _| match &list[0] {
            Value::Atom(v) => {
                v.replace(list[1].clone());
                Ok(list[1].clone())
            }
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );
    env.set(
        "swap!",
        make_fn_val(|list, env| match &list[0] {
            Value::Atom(v) => {
                let mut fn_args = vec![v.borrow().clone()];
                fn_args.extend(list[2..].iter().cloned());
                let result = eval_fn_no_tco(list[1].clone(), fn_args, env)?;
                v.replace(result.clone());
                Ok(result)
            }
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );

    env.set(
        "cons",
        make_fn_val(|args, _| {
            let mut list = args[1].clone().try_into_list_or_vec()?;
            list.insert(0, args[0].clone());
            Ok(Value::List(list, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "concat",
        make_fn_val(|args, _| {
            let mut list = Vec::new();
            for arg in args {
                list.append(&mut arg.clone().try_into_list_or_vec()?);
            }
            Ok(Value::List(list, Box::new(Value::Nil)))
        }),
    );

    env.set(
        "vec",
        make_fn_val(|args, _| {
            Ok(Value::Vec(
                args[0].clone().try_into_list_or_vec()?,
                Box::new(Value::Nil),
            ))
        }),
    );

    env.set(
        "nth",
        make_fn_val(|args, _| match &args[0] {
            Value::List(l, _) | Value::Vec(l, _) => {
                let index_unconverted: i32 = args[1].try_as_number().unwrap();
                let index: usize = index_unconverted
                    .try_into()
                    .map_err(|_| runtime_errors::out_of_bounds(l.len(), index_unconverted))?;

                Ok(l.get(index)
                    .ok_or_else(|| runtime_errors::out_of_bounds(l.len(), index_unconverted))?
                    .clone())
            }
            Value::Nil => Ok(Value::Nil),
            v => Err(runtime_errors::not_a("list", v)),
        }),
    );
    env.set(
        "first",
        make_fn_val(|args, _| match &args[0] {
            Value::List(l, _) | Value::Vec(l, _) if !l.is_empty() => Ok(l[0].clone()),
            Value::List(_, _) | Value::Vec(_, _) | Value::Nil => Ok(Value::Nil),
            v => Err(runtime_errors::not_a("list", v)),
        }),
    );
    env.set(
        "rest",
        make_fn_val(|args, _| match &args[0] {
            Value::List(l, _) | Value::Vec(l, _) if !l.is_empty() => {
                Ok(Value::List(l[1..].to_vec(), Box::new(Value::Nil)))
            }
            Value::List(_, _) | Value::Vec(_, _) | Value::Nil => {
                Ok(Value::List(Vec::new(), Box::new(Value::Nil)))
            }
            v => Err(runtime_errors::not_a("list", v)),
        }),
    );
    env.set("throw", make_fn_val(|args, _| Err(args[0].clone())));

    env.set("apply", Value::HostFn(HostFn::Apply, Box::new(Value::Nil)));
    env.set(
        "map",
        make_fn_val(|args, env| {
            let function = args[0].clone();
            let list = args[1].clone().try_into_list_or_vec()?;
            let mut new_list = Vec::with_capacity(list.len());
            for e in list.into_iter() {
                new_list.push(eval_fn_no_tco(function.clone(), vec![e], env.clone())?);
            }
            Ok(Value::List(new_list, Box::new(Value::Nil)))
        }),
    );

    env.set(
        "nil?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Nil)))),
    );
    env.set(
        "true?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Bool(true))))),
    );
    env.set(
        "false?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Bool(false))))),
    );
    env.set(
        "symbol?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Symbol(_))))),
    );
    env.set(
        "symbol",
        make_fn_val(|args, _| Ok(Value::Symbol(args[0].try_as_str()?.to_string()))),
    );
    env.set(
        "keyword",
        make_fn_val(|args, _| match &args[0] {
            Value::String(s) => Ok(Value::Keyword(format!(
                "{}:{}",
                char::from_u32(0x29E).unwrap(),
                s.to_string()
            ))),
            v @ Value::Keyword(_) => Ok(v.clone()),
            v => Err(runtime_errors::not_a("string or keyword", v)),
        }),
    );
    env.set(
        "keyword?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Keyword(_))))),
    );
    env.set(
        "vector",
        make_fn_val(|list, _| Ok(Value::Vec(list.to_vec(), Box::new(Value::Nil)))),
    );
    env.set(
        "vector?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Vec(_, _))))),
    );
    env.set(
        "sequential?",
        make_fn_val(|args, _| {
            Ok(Value::Bool(matches!(
                &args[0],
                Value::Vec(_, _) | Value::List(_, _)
            )))
        }),
    );
    env.set(
        "hash-map",
        make_fn_val(|args, _| {
            let mut map = HashMap::new();
            let mut args = args.iter();
            ensure_even_args(&args)?;
            while let Some(v) = args.next() {
                map.insert(
                    v.as_hash_map_key()?.to_owned(),
                    args.next().unwrap().clone(),
                );
            }
            Ok(Value::Map(map, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "map?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Map(_, _))))),
    );
    env.set(
        "assoc",
        make_fn_val(|args, _| {
            let mut map = args[0].try_as_map()?.clone();
            let mut args = args.iter().skip(1);
            ensure_even_args(&args)?;
            while let Some(v) = args.next() {
                map.insert(
                    v.as_hash_map_key()?.to_owned(),
                    args.next().unwrap().clone(),
                );
            }
            Ok(Value::Map(map, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "dissoc",
        make_fn_val(|args, _| {
            let mut map = args[0].try_as_map()?.clone();
            for arg in args.iter().skip(1) {
                map.remove(arg.as_hash_map_key()?);
            }
            Ok(Value::Map(map, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "get",
        make_fn_val(|args, _| {
            // TODO: should we just consider Nil to be an empty map in try_as map?
            if matches!(&args[0], Value::Nil) {
                return Ok(Value::Nil);
            }
            let map = args[0].try_as_map()?;
            Ok(map
                .get(args[1].as_hash_map_key()?)
                .cloned()
                .unwrap_or(Value::Nil))
        }),
    );
    env.set(
        "contains?",
        make_fn_val(|args, _| {
            let map = args[0].try_as_map()?;
            Ok(Value::Bool(map.contains_key(args[1].as_hash_map_key()?)))
        }),
    );
    env.set(
        "keys",
        make_fn_val(|args, _| {
            let map = args[0].try_as_map()?;
            Ok(Value::List(
                map.keys()
                    .map(|v| {
                        if v.starts_with(char::from_u32(0x29E).unwrap()) {
                            Value::Keyword(v.clone())
                        } else {
                            Value::String(v.clone())
                        }
                    })
                    .collect(),
                Box::new(Value::Nil),
            ))
        }),
    );
    env.set(
        "vals",
        make_fn_val(|args, _| {
            let map = args[0].try_as_map()?;
            Ok(Value::List(
                map.values().cloned().collect(),
                Box::new(Value::Nil),
            ))
        }),
    );

    // TODO: time-ms, meta, with-meta, fn? string?, number?, seq, and conj

    env.set(
        "time-ms",
        make_fn_val(|_, _| {
            Ok(Value::Number(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i32,
            ))
        }),
    );
    env.set(
        "meta",
        make_fn_val(|args, _| match &args[0] {
            Value::List(_, m)
            | Value::Vec(_, m)
            | Value::Map(_, m)
            | Value::HostFn(_, m)
            | Value::Closure(_, m) => Ok(m.as_ref().clone()),
            v => Err(runtime_errors::not_a(
                "value with metadata (list, vec or function)",
                v,
            )),
        }),
    );
    env.set(
        "with-meta",
        make_fn_val(|args, _| {
            let mut v = args[0].clone();
            match &mut v {
                Value::List(_, m)
                | Value::Vec(_, m)
                | Value::Map(_, m)
                | Value::HostFn(_, m)
                | Value::Closure(_, m) => {
                    *m = Box::new(args[1].clone());
                    Ok(v)
                }
                v => Err(runtime_errors::not_a(
                    "value with metadata (list, vec or function)",
                    v,
                )),
            }
        }),
    );
    env.set(
        "fn?",
        make_fn_val(|args, _| {
            Ok(Value::Bool(
                matches!(&args[0], Value::HostFn(_, _))
                    || matches!(
                        &args[0],
                        Value::Closure(c, _) if !c.is_macro
                    ),
            ))
        }),
    );
    env.set(
        "string?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::String(_))))),
    );
    env.set(
        "number?",
        make_fn_val(|args, _| Ok(Value::Bool(matches!(&args[0], Value::Number(_))))),
    );
    env.set(
        "macro?",
        make_fn_val(|args, _| {
            Ok(Value::Bool(matches!(
                &args[0],
                Value::Closure(c, _) if c.is_macro
            )))
        }),
    );
    env.set(
        "seq",
        make_fn_val(|args, _| match &args[0] {
            Value::List(l, _) | Value::Vec(l, _) if l.is_empty() => Ok(Value::Nil),
            Value::String(s) if s.is_empty() => Ok(Value::Nil),
            Value::Nil => Ok(Value::Nil),
            Value::List(l, _) | Value::Vec(l, _) => {
                Ok(Value::List(l.clone(), Box::new(Value::Nil)))
            }
            Value::String(s) => Ok(Value::List(
                s.chars().map(|c| Value::String(c.into())).collect(),
                Box::new(Value::Nil),
            )),
            v => Err(runtime_errors::not_a("valid argument for seq", v)),
        }),
    );
    env.set(
        "conj",
        make_fn_val(|args, _| match &args[0] {
            Value::List(l, _) => {
                let mut new_list = Vec::new();
                for arg in args[1..].iter().rev() {
                    new_list.push(arg.clone())
                }
                new_list.extend(l.iter().cloned());
                Ok(Value::List(new_list, Box::new(Value::Nil)))
            }
            Value::Vec(l, _) => {
                let mut new_list: Vec<Value> = l.to_vec();
                new_list.extend(args[1..].iter().cloned());
                Ok(Value::Vec(new_list, Box::new(Value::Nil)))
            }
            v => Err(runtime_errors::not_a("list or vec", v)),
        }),
    );
}

fn ensure_even_args(args: &impl ExactSizeIterator) -> RuntimeResult<()> {
    if args.len() % 2 != 0 {
        Err(runtime_errors::error_to_string(format!(
            "expected an even number of arguments, got {}",
            args.len()
        )))
    } else {
        Ok(())
    }
}
