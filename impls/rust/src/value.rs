use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::HashMap,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use crate::{
    env::Env,
    printer::pr_str,
    runtime_errors::{RuntimeError, RuntimeResult},
};
#[derive(Clone)]
pub struct Closure {
    pub ast: Value,
    pub params: Vec<Value>,
    pub env: Rc<RefCell<Env>>,
    pub is_macro: bool,
}

type MalFunction = fn(&[Value], Rc<RefCell<Env>>) -> RuntimeResult<Value>;

#[derive(Clone)]
pub enum Value {
    List(Vec<Value>),
    Vec(Vec<Value>),
    Map(HashMap<String, Value>),
    Number(i32),
    Symbol(String),
    Keyword(String),
    String(String),
    Fn(MalFunction),
    Closure(Rc<Closure>),
    Eval(Rc<RefCell<Env>>),
    Nil,
    Bool(bool),
    Atom(Rc<RefCell<Value>>),
}

impl Value {
    pub fn into_hash_map_key(self) -> Result<String, Self> {
        match self {
            Value::Keyword(s) | Value::String(s) => Ok(s),
            _ => Err(self),
        }
    }
    pub fn into_env_map_key(self) -> RuntimeResult<String> {
        match self {
            Value::Symbol(s) => Ok(s),
            _ => Err(RuntimeError::InvalidMapKey(self.to_string())),
        }
    }

    pub fn into_list(self) -> Vec<Value> {
        match self {
            Value::List(l) => l,
            _ => unreachable!(),
        }
    }
    pub fn try_as_list_or_vec(&self) -> Option<&[Value]> {
        match self {
            Value::List(l) | Value::Vec(l) => Some(l),
            _ => None,
        }
    }
    pub fn try_into_list_or_vec(self) -> Option<Vec<Value>> {
        match self {
            Value::List(l) | Value::Vec(l) => Some(l),
            _ => None,
        }
    }
    pub fn value_to_string(&self, readably: bool) -> String {
        let mut buf = String::new();
        pr_str(self, &mut buf, readably).unwrap();
        buf
    }
    pub fn try_as_str(&self) -> Option<&str> {
        match self {
            Value::String(str) => Some(str),
            _ => None,
        }
    }
    pub fn try_as_number(&self) -> Option<i32> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
    fn deref_atom_recursively(&self) -> Self {
        let mut curr = self.clone();
        while let Value::Atom(v) = curr {
            curr = v.borrow().clone();
        }
        curr
    }
}

impl Add for &Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Sub for &Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Mul for &Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Div for &Value {
    type Output = RuntimeResult<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Number(a / b))
                }
            }
            _ => todo!("value type unsupported"),
        }
    }
}
impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Some(a.cmp(b)),
            _ => None,
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.deref_atom_recursively(),
            other.deref_atom_recursively(),
        ) {
            (Value::List(a), Value::List(b))
            | (Value::Vec(a), Value::Vec(b))
            | (Value::List(a), Value::Vec(b))
            | (Value::Vec(a), Value::List(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Keyword(a), Value::Keyword(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Fn(a), Value::Fn(b)) => std::ptr::eq(&a, &b),
            (Value::Closure(a), Value::Closure(b)) => Rc::ptr_eq(&a, &b),
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Eval(_), Value::Eval(_)) => true,
            _ => false,
        }
    }
}
impl Eq for Value {}
