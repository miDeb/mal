use std::{cell::RefCell, convert::TryInto, rc::Rc};

use rustc_hash::FxHashMap;

use crate::{
    env::Env,
    eval_fn_no_tco,
    printer::pr_str,
    reader::{read_str, ParseError},
    runtime_errors::{self, error_to_string_with_ctx, RuntimeResult},
    value::{HostFn, MalFnPtr, Value},
};

pub fn init_env(env: &mut Env) {
    fn make_fn_val(
        f: fn(std::vec::IntoIter<Value>, Rc<RefCell<Env>>) -> RuntimeResult<Value>,
    ) -> Value {
        Value::HostFn(HostFn::ByPtr(MalFnPtr(f)), Box::new(Value::Nil))
    }
    env.set(
        "+",
        make_fn_val(|mut args, _| Ok(args.next().unwrap() + args.next().unwrap())),
    );
    env.set(
        "-",
        make_fn_val(|mut args, _| Ok(args.next().unwrap() - args.next().unwrap())),
    );
    env.set(
        "*",
        make_fn_val(|mut args, _| Ok(args.next().unwrap() * args.next().unwrap())),
    );
    env.set(
        "/",
        make_fn_val(|mut args, _| args.next().unwrap() / args.next().unwrap()),
    );
    env.set(
        "pr-str",
        make_fn_val(|args, _| {
            let mut string = String::new();
            for (i, item) in args.enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(&item, &mut string, true).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "str",
        make_fn_val(|args, _| {
            let mut string = String::new();
            for item in args {
                pr_str(&item, &mut string, false).unwrap();
            }
            Ok(Value::String(string))
        }),
    );
    env.set(
        "prn",
        make_fn_val(|args, _| {
            let mut string = String::new();
            for (i, item) in args.enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(&item, &mut string, true).unwrap();
            }
            println!("{}", string);
            Ok(Value::Nil)
        }),
    );
    env.set(
        "println",
        make_fn_val(|args, _| {
            let mut string = String::new();
            for (i, item) in args.enumerate() {
                if i != 0 {
                    string.push(' ');
                }
                pr_str(&item, &mut string, false).unwrap();
            }
            println!("{}", string);
            Ok(Value::Nil)
        }),
    );
    env.set(
        "list",
        make_fn_val(|args, _| Ok(Value::List(args.collect(), Box::new(Value::Nil)))),
    );
    env.set(
        "list?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::List(_, _)
            )))
        }),
    );
    env.set(
        "empty?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(
                args.next()
                    .unwrap()
                    .try_as_list_or_vec()
                    .map(|l| l.is_empty())
                    .unwrap_or(true),
            ))
        }),
    );
    env.set(
        "count",
        make_fn_val(|mut args, _| {
            Ok(Value::Number(
                args.next()
                    .unwrap()
                    .try_as_list_or_vec()
                    .map(|l| l.len())
                    .unwrap_or(0) as i32,
            ))
        }),
    );
    env.set(
        "=",
        make_fn_val(|mut args, _| Ok(Value::Bool(args.next().unwrap() == args.next().unwrap()))),
    );
    env.set(
        "<",
        make_fn_val(|mut args, _| Ok(Value::Bool(args.next().unwrap() < args.next().unwrap()))),
    );
    env.set(
        ">",
        make_fn_val(|mut args, _| Ok(Value::Bool(args.next().unwrap() > args.next().unwrap()))),
    );
    env.set(
        "<=",
        make_fn_val(|mut args, _| Ok(Value::Bool(args.next().unwrap() <= args.next().unwrap()))),
    );
    env.set(
        ">=",
        make_fn_val(|mut args, _| Ok(Value::Bool(args.next().unwrap() >= args.next().unwrap()))),
    );

    env.set(
        "read-string",
        make_fn_val(
            |mut args, _| match read_str(args.next().unwrap().try_as_str()?) {
                Ok(v) => Ok(v),
                Err(ParseError::EmptyInput) => Ok(Value::Nil),
                Err(e) => Err(error_to_string_with_ctx("parsing failed", e)),
            },
        ),
    );
    env.set(
        "slurp",
        make_fn_val(|mut args, _| {
            let file = args.next().unwrap().try_into_string()?;
            std::fs::read_to_string(&file)
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
        make_fn_val(|mut args, _| Ok(Value::Atom(Rc::new(RefCell::new(args.next().unwrap()))))),
    );
    env.set(
        "atom?",
        make_fn_val(|mut args, _| Ok(Value::Bool(matches!(args.next().unwrap(), Value::Atom(_))))),
    );
    env.set(
        "deref",
        make_fn_val(|mut args, _| match &args.next().unwrap() {
            Value::Atom(v) => Ok(v.borrow().clone()),
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );
    env.set(
        "reset!",
        make_fn_val(|mut args, _| match &args.next().unwrap() {
            Value::Atom(v) => {
                let arg = args.next().unwrap();
                v.replace(arg.clone());
                Ok(arg)
            }
            v => Err(runtime_errors::not_a("atom", v)),
        }),
    );
    env.set(
        "swap!",
        make_fn_val(|mut args, env| match args.next().unwrap() {
            Value::Atom(v) => {
                let mut fn_args = vec![v.borrow().clone()];
                let fun = args.next().unwrap();
                fn_args.extend(args);
                let result = eval_fn_no_tco(fun, fn_args, env)?;
                v.replace(result.clone());
                Ok(result)
            }
            v => Err(runtime_errors::not_a("atom", &v)),
        }),
    );

    env.set(
        "cons",
        make_fn_val(|mut args, _| {
            let mut list = Vec::with_capacity(args.len() + 1);
            list.push(args.next().unwrap());
            list.append(&mut args.next().unwrap().try_into_list_or_vec()?);
            Ok(Value::List(list, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "concat",
        make_fn_val(|args, _| {
            let mut list = Vec::new();
            for arg in args {
                list.append(&mut arg.try_into_list_or_vec()?);
            }
            Ok(Value::List(list, Box::new(Value::Nil)))
        }),
    );

    env.set(
        "vec",
        make_fn_val(|mut args, _| {
            Ok(Value::Vec(
                args.next().unwrap().try_into_list_or_vec()?,
                Box::new(Value::Nil),
            ))
        }),
    );

    env.set(
        "nth",
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::List(mut l, _) | Value::Vec(mut l, _) => {
                let index_unconverted: i32 = args.next().unwrap().try_as_number().unwrap();
                let index: usize = index_unconverted
                    .try_into()
                    .map_err(|_| runtime_errors::out_of_bounds(l.len(), index_unconverted))?;

                if index >= l.len() {
                    Err(runtime_errors::out_of_bounds(l.len(), index_unconverted))
                } else {
                    Ok(l.swap_remove(index))
                }
            }
            Value::Nil => Ok(Value::Nil),
            v => Err(runtime_errors::not_a("list", &v)),
        }),
    );
    env.set(
        "first",
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::List(mut l, _) | Value::Vec(mut l, _) if !l.is_empty() => Ok(l.swap_remove(0)),
            Value::List(_, _) | Value::Vec(_, _) | Value::Nil => Ok(Value::Nil),
            v => Err(runtime_errors::not_a("list", &v)),
        }),
    );
    env.set(
        "rest",
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::List(mut l, _) | Value::Vec(mut l, _) if !l.is_empty() => {
                l.remove(0);
                Ok(Value::List(l, Box::new(Value::Nil)))
            }
            Value::List(_, _) | Value::Vec(_, _) | Value::Nil => {
                Ok(Value::List(Vec::new(), Box::new(Value::Nil)))
            }
            v => Err(runtime_errors::not_a("list", &v)),
        }),
    );
    env.set(
        "throw",
        make_fn_val(|mut args, _| Err(args.next().unwrap())),
    );

    env.set("apply", Value::HostFn(HostFn::Apply, Box::new(Value::Nil)));
    env.set(
        "map",
        make_fn_val(|mut args, env| {
            let function = args.next().unwrap();
            let list = args.next().unwrap().try_into_list_or_vec()?;
            let mut new_list = Vec::with_capacity(list.len());
            for e in list.into_iter() {
                new_list.push(eval_fn_no_tco(function.clone(), vec![e], env.clone())?);
            }
            Ok(Value::List(new_list, Box::new(Value::Nil)))
        }),
    );

    env.set(
        "nil?",
        make_fn_val(|mut args, _| Ok(Value::Bool(matches!(args.next().unwrap(), Value::Nil)))),
    );
    env.set(
        "true?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Bool(true)
            )))
        }),
    );
    env.set(
        "false?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Bool(false)
            )))
        }),
    );
    env.set(
        "symbol?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Symbol(_)
            )))
        }),
    );
    env.set(
        "symbol",
        make_fn_val(|mut args, _| {
            Ok(Value::Symbol(
                args.next().unwrap().try_as_str()?.to_string(),
            ))
        }),
    );
    env.set(
        "keyword",
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::String(s) => Ok(Value::Keyword(format!(
                "{}:{}",
                char::from_u32(0x29E).unwrap(),
                s
            ))),
            v @ Value::Keyword(_) => Ok(v),
            v => Err(runtime_errors::not_a("string or keyword", &v)),
        }),
    );
    env.set(
        "keyword?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Keyword(_)
            )))
        }),
    );
    env.set(
        "vector",
        make_fn_val(|args, _| Ok(Value::Vec(args.collect(), Box::new(Value::Nil)))),
    );
    env.set(
        "vector?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Vec(_, _)
            )))
        }),
    );
    env.set(
        "sequential?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Vec(_, _) | Value::List(_, _)
            )))
        }),
    );
    env.set(
        "hash-map",
        make_fn_val(|mut args, _| {
            let mut map = FxHashMap::default();
            ensure_even_args(&args)?;
            while let Some(v) = args.next() {
                map.insert(v.as_hash_map_key()?.to_owned(), args.next().unwrap());
            }
            Ok(Value::Map(map, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "map?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Map(_, _)
            )))
        }),
    );
    env.set(
        "assoc",
        make_fn_val(|mut args, _| {
            let mut map = args.next().unwrap().try_into_map()?;
            ensure_even_args(&args)?;
            while let Some(v) = args.next() {
                map.insert(v.as_hash_map_key()?.to_owned(), args.next().unwrap());
            }
            Ok(Value::Map(map, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "dissoc",
        make_fn_val(|mut args, _| {
            let mut map = args.next().unwrap().try_into_map()?;
            for arg in args.skip(1) {
                map.remove(arg.as_hash_map_key()?);
            }
            Ok(Value::Map(map, Box::new(Value::Nil)))
        }),
    );
    env.set(
        "get",
        make_fn_val(|mut args, _| {
            // TODO: should we just consider Nil to be an empty map in try_as map?
            let map = args.next().unwrap();
            if matches!(&map, Value::Nil) {
                return Ok(Value::Nil);
            }
            let mut map = map.try_into_map()?;
            // We can remove from the map because the map is not used anywhere else.
            // If we ever switch to refcounting values or even GC this would not likely be the best solution.
            Ok(map
                .remove(args.next().unwrap().as_hash_map_key()?)
                .unwrap_or(Value::Nil))
        }),
    );
    env.set(
        "contains?",
        make_fn_val(|mut args, _| {
            let map = args.next().unwrap().try_into_map()?;
            Ok(Value::Bool(
                map.contains_key(args.next().unwrap().as_hash_map_key()?),
            ))
        }),
    );
    env.set(
        "keys",
        make_fn_val(|mut args, _| {
            let map = args.next().unwrap().try_into_map()?;
            Ok(Value::List(
                map.into_keys()
                    .map(|v| {
                        if v.starts_with(char::from_u32(0x29E).unwrap()) {
                            Value::Keyword(v)
                        } else {
                            Value::String(v)
                        }
                    })
                    .collect(),
                Box::new(Value::Nil),
            ))
        }),
    );
    env.set(
        "vals",
        make_fn_val(|mut args, _| {
            let map = args.next().unwrap().try_into_map()?;
            Ok(Value::List(
                map.into_values().collect(),
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
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::List(_, m)
            | Value::Vec(_, m)
            | Value::Map(_, m)
            | Value::HostFn(_, m)
            | Value::Closure(_, m) => Ok(*m),
            v => Err(runtime_errors::not_a(
                "value with metadata (list, vec or function)",
                &v,
            )),
        }),
    );
    env.set(
        "with-meta",
        make_fn_val(|mut args, _| {
            let mut v = args.next().unwrap();
            match &mut v {
                Value::List(_, m)
                | Value::Vec(_, m)
                | Value::Map(_, m)
                | Value::HostFn(_, m)
                | Value::Closure(_, m) => {
                    *m = Box::new(args.next().unwrap());
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
        make_fn_val(|mut args, _| {
            let arg = args.next().unwrap();
            Ok(Value::Bool(
                matches!(arg, Value::HostFn(_, _))
                    || matches!(
                        arg,
                        Value::Closure(c, _) if !c.is_macro
                    ),
            ))
        }),
    );
    env.set(
        "string?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::String(_)
            )))
        }),
    );
    env.set(
        "number?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Number(_)
            )))
        }),
    );
    env.set(
        "macro?",
        make_fn_val(|mut args, _| {
            Ok(Value::Bool(matches!(
                args.next().unwrap(),
                Value::Closure(c, _) if c.is_macro
            )))
        }),
    );
    env.set(
        "seq",
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::List(l, _) | Value::Vec(l, _) if l.is_empty() => Ok(Value::Nil),
            Value::String(s) if s.is_empty() => Ok(Value::Nil),
            Value::Nil => Ok(Value::Nil),
            Value::List(l, _) | Value::Vec(l, _) => Ok(Value::List(l, Box::new(Value::Nil))),
            Value::String(s) => Ok(Value::List(
                s.chars().map(|c| Value::String(c.into())).collect(),
                Box::new(Value::Nil),
            )),
            v => Err(runtime_errors::not_a("valid argument for seq", &v)),
        }),
    );
    env.set(
        "conj",
        make_fn_val(|mut args, _| match args.next().unwrap() {
            Value::List(l, _) => {
                let mut new_list: Vec<Value> = args.rev().collect();
                new_list.extend(l.into_iter());
                Ok(Value::List(new_list, Box::new(Value::Nil)))
            }
            Value::Vec(l, _) => {
                let mut new_list: Vec<Value> = l.to_vec();
                new_list.extend(args);
                Ok(Value::Vec(new_list, Box::new(Value::Nil)))
            }
            v => Err(runtime_errors::not_a("list or vec", &v)),
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
