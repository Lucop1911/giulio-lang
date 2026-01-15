use crate::interpreter::obj::Object;

// Method only
pub fn bisempty_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(s)) => {
            Ok(Object::Boolean(s.is_empty()))
        }
        Some(Object::Array(arr)) => {
            Ok(Object::Boolean(arr.is_empty()))
        }
        Some(Object::Hash(hash)) => {
            Ok(Object::Boolean(hash.is_empty()))
        }
        _ => Err("Invalid arguments to is_empty()".to_string())
    }
}

pub fn blen_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(s)) => Ok(Object::Integer(s.len() as i64)),
        Some(Object::Array(arr)) => Ok(Object::Integer(arr.len() as i64)),
        Some(Object::Hash(hash)) => {
            Ok(Object::Integer(hash.len() as i64))
        }
        _ => Err("len() requires a string or array".to_string()),
    }
}