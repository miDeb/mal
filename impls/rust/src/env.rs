#![allow(dead_code)]
use crate::runtime_errors::{RuntimeError, RuntimeResult};
use crate::Value;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub struct Env {
    pub data: HashMap<String, Value>,
    outer: Option<Rc<RefCell<Env>>>,
}

impl Env {
    pub fn new(outer: Option<Rc<RefCell<Env>>>) -> Self {
        Self {
            data: HashMap::new(),
            outer,
        }
    }

    pub fn set(&mut self, key: impl Into<String>, value: Value) {
        self.data.insert(key.into(), value);
    }

    pub fn find(env: &Rc<RefCell<Env>>, key: &str) -> Option<Rc<RefCell<Env>>> {
        if env.as_ref().borrow().data.contains_key(key) {
            Some(env.clone())
        } else if let Some(outer) = &env.as_ref().borrow().outer {
            Self::find(outer, key)
        } else {
            None
        }
    }

    pub fn get(env: &Rc<RefCell<Env>>, key: &str) -> RuntimeResult<Value> {
        Ok(Self::find(env, key)
            .ok_or_else(|| RuntimeError::NotFoundInEnv(key.into()))?
            .as_ref()
            .borrow()
            .data
            .get(key)
            .unwrap()
            .clone())
    }
}
