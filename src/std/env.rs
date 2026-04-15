use crate::runtime::obj::Object;
use crate::runtime::runtime_errors::RuntimeError;
use std::env::args;

pub fn env_args(_args: Vec<Object>) -> Result<Object, RuntimeError> {
    let args: Vec<Object> = args().skip(1).map(Object::String).collect();
    Ok(Object::Array(args))
}
