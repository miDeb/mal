use std::fmt::Display;

pub enum RuntimeError {
    FnNotFound(String),
    NotAFunction(String),
    DivisionByZero,
    NotFoundInEnv(String),
    InvalidMapKey(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::FnNotFound(fun) => write!(f, "Function '{}' not found", fun),
            RuntimeError::NotAFunction(no_fun) => write!(f, "'{}' is not a function", no_fun),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::NotFoundInEnv(v) => write!(f, "'{}' not found in the environment", v),
            RuntimeError::InvalidMapKey(k) => write!(f, "'{}' is not a valid map key", k),
        }
    }
}
pub type RuntimeResult<T> = Result<T, RuntimeError>;
