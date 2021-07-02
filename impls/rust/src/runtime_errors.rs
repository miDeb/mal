use std::fmt::Display;

pub enum RuntimeError {
    FnNotFound(String),
    NotAFunction(String),
    DivisionByZero,
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::FnNotFound(fun) => write!(f, "Function '{}' not found", fun),
            RuntimeError::NotAFunction(no_fun) => write!(f, "'{}' is not a function", no_fun),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}
pub type RuntimeResult<T> = Result<T, RuntimeError>;
