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

// Method only
pub fn bremove_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Hash(mut hash)), Some(key)) => {
            match &key {
                Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                    hash.remove(&key);
                    Ok(Object::Hash(hash))
                }
                _ => Err(format!("{} is not hashable", key.type_name())),
            }
        }
        (Some(Object::Array(mut vec)), Some(Object::Integer(idx))) => {
            let i = idx as isize;
            if i < 0 || i as usize >= vec.len() {
                return Err("Index out of bounds".to_string());
            }
            let _ = vec.remove(i as usize);
            Ok(Object::Array(vec))
        }
        _ => Err("Invalid arguments to remove(hash, key)".to_string()),
    }
}