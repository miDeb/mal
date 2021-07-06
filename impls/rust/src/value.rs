use std::{
    cell::RefCell,
    cmp::Ordering,
    fmt,
    ops::{Add, Div, Mul, Sub},
    rc::Rc,
};

use rustc_hash::FxHashMap;
use rustyline::Editor;

use crate::{
    env::Env,
    runtime_errors::{self, RuntimeResult},
};
#[derive(Clone, Debug)]
pub struct Closure {
    pub ast: Value,
    pub params: Vec<Value>,
    pub env: Rc<RefCell<Env>>,
    pub is_macro: bool,
}

#[derive(Clone)]
pub struct MalFnPtr(pub fn(std::vec::IntoIter<Value>, Rc<RefCell<Env>>) -> RuntimeResult<Value>);

impl fmt::Debug for MalFnPtr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(self.0 as *const ()), f)
    }
}

#[derive(Clone, Debug)]
pub enum HostFn {
    ByPtr(MalFnPtr),
    Eval(Rc<RefCell<Env>>),
    Apply,
    ReadLine(Rc<RefCell<Editor<()>>>),
}

impl PartialEq for HostFn {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (HostFn::ByPtr(a), HostFn::ByPtr(b)) => std::ptr::eq(&a, &b),
            (HostFn::Eval(_), HostFn::Eval(_)) => true,
            (HostFn::Apply, HostFn::Apply) => true,
            _ => false,
        }
    }
}

impl Eq for HostFn {}

pub type Meta = Box<Value>;

#[derive(Clone, Debug)]
pub enum Value {
    List(Vec<Value>, Meta),
    Vec(Vec<Value>, Meta),
    Map(FxHashMap<String, Value>, Meta),
    Number(i32),
    Symbol(String),
    Keyword(String),
    String(String),
    HostFn(HostFn, Meta),
    Closure(Rc<Closure>, Meta),
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
    pub fn as_hash_map_key(&self) -> RuntimeResult<&str> {
        match self {
            Value::Keyword(s) | Value::String(s) => Ok(s),
            v => Err(runtime_errors::not_a("hash map key (keyword or string)", v)),
        }
    }
    pub fn try_into_env_map_key(self) -> RuntimeResult<String> {
        match self {
            Value::Symbol(s) => Ok(s),
            _ => Err(runtime_errors::not_a("map key (symbol)", &self)),
        }
    }

    pub fn into_list(self) -> Vec<Value> {
        match self {
            Value::List(l, _) => l,
            _ => unreachable!(),
        }
    }
    pub fn try_as_list_or_vec(&self) -> Option<&[Value]> {
        match self {
            Value::List(l, _) | Value::Vec(l, _) => Some(l),
            _ => None,
        }
    }
    pub fn try_into_list_or_vec(self) -> RuntimeResult<Vec<Value>> {
        match self {
            Value::List(l, _) | Value::Vec(l, _) => Ok(l),
            v => Err(runtime_errors::not_a("list or vec", &v)),
        }
    }
    /*pub fn value_to_string(&self, readably: bool) -> String {
        let mut buf = String::new();
        pr_str(self, &mut buf, readably).unwrap();
        buf
    }*/
    pub fn try_as_str(&self) -> RuntimeResult<&str> {
        match self {
            Value::String(str) => Ok(str),
            v => Err(runtime_errors::not_a("string", v)),
        }
    }
    pub fn try_into_string(self) -> RuntimeResult<String> {
        match self {
            Value::String(str) => Ok(str),
            v => Err(runtime_errors::not_a("string", &v)),
        }
    }
    pub fn try_as_number(&self) -> Option<i32> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }
    /*pub fn try_as_map(&self) -> RuntimeResult<&FxHashMap<String, Value>> {
        match self {
            Value::Map(m, _) => Ok(m),
            v => Err(runtime_errors::not_a("hash map", v)),
        }
    }*/
    pub fn try_into_map(self) -> RuntimeResult<FxHashMap<String, Value>> {
        match self {
            Value::Map(m, _) => Ok(m),
            v => Err(runtime_errors::not_a("hash map", &v)),
        }
    }
    fn deref_atom_recursively(mut self) -> Self {
        while let Value::Atom(v) = self {
            self = v.borrow().clone();
        }
        self
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Sub for Value {
    type Output = Value;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Mul for Value {
    type Output = Value;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => todo!("value type unsupported"),
        }
    }
}
impl Div for Value {
    type Output = RuntimeResult<Value>;

    fn div(self, rhs: Self) -> Self::Output {
        match (self.deref_atom_recursively(), rhs.deref_atom_recursively()) {
            (Value::Number(a), Value::Number(b)) => {
                if b == 0 {
                    Err(runtime_errors::error_to_string("division by zero"))
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
            self.clone().deref_atom_recursively(),
            other.clone().deref_atom_recursively(),
        ) {
            (Value::List(a, _), Value::List(b, _))
            | (Value::Vec(a, _), Value::Vec(b, _))
            | (Value::List(a, _), Value::Vec(b, _))
            | (Value::Vec(a, _), Value::List(b, _)) => a == b,
            (Value::Map(a, _), Value::Map(b, _)) => a == b,
            (Value::Symbol(a), Value::Symbol(b)) => a == b,
            (Value::Keyword(a), Value::Keyword(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::HostFn(a, _), Value::HostFn(b, _)) => a == b,
            (Value::Closure(a, _), Value::Closure(b, _)) => Rc::ptr_eq(&a, &b),
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Nil, Value::Nil) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Value {}
