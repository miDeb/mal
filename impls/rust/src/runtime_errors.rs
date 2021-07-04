use crate::value::Value;

pub fn not_a(not: &str, but: &Value) -> Value {
    Value::String(format!("expected {}, got {}", not, but))
}

pub fn out_of_bounds(len: usize, val: i32) -> Value {
    Value::String(format!(
        "index out of bounds: length is {}, got {}",
        len, val
    ))
}

pub fn error_to_string_with_ctx(ctx: impl AsRef<str>, e: impl ToString) -> Value {
    Value::String(format!("{}: {}", ctx.as_ref(), e.to_string()))
}
pub fn error_to_string(e: impl ToString) -> Value {
    Value::String(e.to_string())
}

pub type RuntimeResult<T> = Result<T, Value>;
