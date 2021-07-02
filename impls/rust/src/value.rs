use std::collections::HashMap;

#[derive(Debug)]
pub enum Value {
    List(Vec<Value>),
    Vec(Vec<Value>),
    Map(HashMap<String, Value>),
    Number(i32),
    Symbol(String),
    Keyword(String),
    String(String),
}

impl Value {
    pub fn into_hash_map_key(self) -> Option<String> {
        match self {
            Value::Keyword(s) | Value::String(s) => Some(s),
            _ => None,
        }
    }
}
