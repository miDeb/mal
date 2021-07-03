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

    pub fn new_with_binds(
        outer: Option<Rc<RefCell<Env>>>,
        mut binds: impl Iterator<Item = Value>,
        mut exprs: impl Iterator<Item = Value>,
    ) -> RuntimeResult<Self> {
        let mut env = Self {
            outer,
            data: HashMap::new(),
        };
        while let Some(key) = binds.next() {
            if matches!(&key, Value::Symbol(s) if s == "&") {
                let key = binds.next().unwrap();
                let values = exprs.collect();
                env.set(key.into_env_map_key()?, Value::List(values));
                break;
            } else {
                env.set(key.into_env_map_key()?, exprs.next().unwrap())
            }
        }
        Ok(env)
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
