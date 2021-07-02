use std::{
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use crate::runtime_errors::{RuntimeError, RuntimeResult};

#[derive(Clone)]
pub enum Value {
    List(Vec<Value>),
    Vec(Vec<Value>),
    Map(HashMap<String, Value>),
    Number(i32),
    Symbol(String),
    Keyword(String),
    String(String),
    Fn(Rc<dyn Fn(&[Value]) -> RuntimeResult<Value>>),
}

impl Value {
    pub fn into_hash_map_key(self) -> Option<String> {
        match self {
            Value::Keyword(s) | Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn into_list(self) -> Vec<Value> {
        match self {
            Value::List(l) => l,
            _ => unreachable!(),
        }
    }
    pub fn as_fn(&self) -> RuntimeResult<&dyn Fn(&[Value]) -> RuntimeResult<Value>> {
        match self {
            Value::Fn(f) => Ok(f.as_ref()),
            no_fun => Err(RuntimeError::NotAFunction(no_fun.to_string())),
        }
    }
}

impl Add for &Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Sub for &Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Mul for &Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Div for &Value {
    type Output = RuntimeResult<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => todo!("value type unsupported"),
        }
    }
}
