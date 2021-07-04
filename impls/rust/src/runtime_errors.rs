use std::fmt::Display;

use crate::reader::ParseError;

#[derive(Debug)]
pub enum RuntimeError {
    FnNotFound(String),
    NotAFunction(String),
    NotAList(String),
    NotAnAtom(String),
    DivisionByZero,
    NotFoundInEnv(String),
    InvalidMapKey(String),
    ParseError(ParseError),
    IoError(std::io::Error),
    Index,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::FnNotFound(fun) => write!(f, "Function '{}' not found", fun),
            RuntimeError::NotAFunction(no_fun) => write!(f, "'{}' is not a function", no_fun),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::NotFoundInEnv(v) => write!(f, "'{}' not found in the environment", v),
            RuntimeError::InvalidMapKey(k) => write!(f, "'{}' is not a valid map key", k),
            RuntimeError::ParseError(e) => write!(f, "parsing failed: {}", e),
            RuntimeError::IoError(e) => write!(f, "{}", e),
            RuntimeError::NotAnAtom(no_atom) => write!(f, "'{}' is not an atom", no_atom),
            RuntimeError::Index => write!(f, "invalid index"),
            RuntimeError::NotAList(no_list) => write!(f, "'{}' is not a list", no_list),
        }
    }
}
pub type RuntimeResult<T> = Result<T, RuntimeError>;
