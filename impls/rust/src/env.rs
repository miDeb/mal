use crate::Value;
use std::{collections::HashMap, rc::Rc};

pub struct Env {
    pub map: HashMap<String, Value>,
}

impl Env {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(
            "+".to_string(),
            Value::Fn(Rc::new(|list| Ok(&list[0] + &list[1]))),
        );
        map.insert(
            "-".to_string(),
            Value::Fn(Rc::new(|list| Ok(&list[0] - &list[1]))),
        );
        map.insert(
            "*".to_string(),
            Value::Fn(Rc::new(|list| Ok(&list[0] * &list[1]))),
        );
        map.insert(
            "/".to_string(),
            Value::Fn(Rc::new(|list| &list[0] / &list[1])),
        );
        Self { map }
    }
}
