use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{Display, Write},
};

use crate::value::Value;

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        pr_str(self, f, true)
    }
}

pub fn pr_str(value: &Value, f: &mut impl Write, readably: bool) -> std::fmt::Result {
    match value {
        Value::Number(n) => write!(f, "{}", n),
        Value::Symbol(name) => write!(f, "{}", name),
        Value::List(list) => {
            write!(f, "(")?;
            write_list(f, list, readably)?;
            write!(f, ")")
        }
        Value::Vec(list) => {
            write!(f, "[")?;
            write_list(f, list, readably)?;
            write!(f, "]")
        }
        Value::Keyword(name) => write!(
            f,
            "{}",
            name.strip_prefix(char::from_u32(0x29E).unwrap()).unwrap()
        ),
        Value::String(value) => {
            if readably {
                write!(f, "{:?}", value)
            } else {
                write!(f, "{}", value)
            }
        }
        Value::Map(map) => write_map(f, map, readably),
        Value::Fn(_) | Value::Closure(_) | Value::Eval(_) => write!(f, "#<function>"),
        Value::Nil => write!(f, "nil"),
        Value::Bool(b) => write!(f, "{}", b),
        Value::Atom(atom) => write!(f, "(atom {})", RefCell::borrow(atom)),
    }
}

fn write_list(f: &mut impl Write, list: &[Value], readably: bool) -> std::fmt::Result {
    for (i, elem) in list.iter().enumerate() {
        if i != 0 {
            f.write_char(' ')?;
        }
        pr_str(elem, f, readably)?;
    }
    Ok(())
}

fn write_map(f: &mut impl Write, map: &HashMap<String, Value>, readably: bool) -> std::fmt::Result {
    write!(f, "{{")?;
    for (i, (key, value)) in map.iter().enumerate() {
        if i != 0 {
            write!(f, " ")?;
        }
        if let Some(keyword) = key.strip_prefix(char::from_u32(0x29E).unwrap()) {
            write!(f, "{} ", keyword)?;
        } else {
            write!(f, "{:?} ", key)?;
        }
        pr_str(value, f, readably)?;
    }
    write!(f, "}}")
}
