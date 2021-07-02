use std::{collections::HashMap, fmt::Write};

use crate::value::Value;

pub fn pr_str(value: &Value, writer: &mut impl Write) {
    match value {
        Value::Number(n) => write!(writer, "{}", n).unwrap(),
        Value::Symbol(name) => write!(writer, "{}", name).unwrap(),
        Value::List(list) => {
            write!(writer, "(").unwrap();
            write_list(writer, list);
            write!(writer, ")").unwrap();
        }
        Value::Vec(list) => {
            write!(writer, "[").unwrap();
            write_list(writer, list);
            write!(writer, "]").unwrap();
        }
        Value::Keyword(name) => write!(
            writer,
            "{}",
            name.strip_prefix(char::from_u32(0x29E).unwrap()).unwrap()
        )
        .unwrap(),
        Value::String(value) => write!(writer, "{:?}", value).unwrap(),
        Value::Map(map) => write_map(writer, map),
    }
}

fn write_list(writer: &mut impl Write, list: &[Value]) {
    for (i, elem) in list.iter().enumerate() {
        if i != 0 {
            write!(writer, " ").unwrap();
        }
        pr_str(elem, writer);
    }
}

fn write_map(writer: &mut impl Write, map: &HashMap<String, Value>) {
    write!(writer, "{{").unwrap();
    for (i, (key, value)) in map.iter().enumerate() {
        if i != 0 {
            write!(writer, " ").unwrap();
        }
        if let Some(keyword) = key.strip_prefix(char::from_u32(0x29E).unwrap()) {
            write!(writer, "{} ", keyword).unwrap();
        } else {
            write!(writer, "{:?} ", key).unwrap();
        }
        pr_str(value, writer);
    }
    write!(writer, "}}").unwrap();
}
