use std::{
    collections::HashMap,
    fmt::{Display, Write},
};

use crate::value::Value;

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Symbol(name) => write!(f, "{}", name),
            Value::List(list) => {
                write!(f, "(")?;
                write_list(f, list)?;
                write!(f, ")")
            }
            Value::Vec(list) => {
                write!(f, "[")?;
                write_list(f, list)?;
                write!(f, "]")
            }
            Value::Keyword(name) => write!(
                f,
                "{}",
                name.strip_prefix(char::from_u32(0x29E).unwrap()).unwrap()
            ),
            Value::String(value) => write!(f, "{:?}", value),
            Value::Map(map) => write_map(f, map),
            Value::Fn(_) => write!(f, "[function]"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

fn write_list(f: &mut impl Write, list: &[Value]) -> std::fmt::Result {
    for (i, elem) in list.iter().enumerate() {
        if i != 0 {
            write!(f, " {}", elem)?;
        } else {
            write!(f, "{}", elem)?;
        }
    }
    Ok(())
}

fn write_map(f: &mut impl Write, map: &HashMap<String, Value>) -> std::fmt::Result {
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
        write!(f, "{}", value)?;
    }
    write!(f, "}}")
}
