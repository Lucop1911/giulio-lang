use num_bigint::ToBigInt;
use num_traits::ToPrimitive;

use crate::interpreter::obj::Object;

// Method only
pub fn btostring_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(obj) => Ok(Object::String(format!("{}", obj))),
        _ => Err("to_string() expects an argument".to_string()),
    }
}

// Method only
pub fn btoint_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => match s.trim().parse::<i64>() {
            Ok(n) => Ok(Object::Integer(n)),
            Err(_) => Err("Cannot convert string to int".to_string()),
        },
        Some(Object::Float(n)) => match n.to_i64() {
            Some(n) => Ok(Object::Integer(n)),
            None => match n.to_bigint() {
                Some(big) => Ok(Object::BigInteger(big)),
                None => Err("Unable to convert this float exactly to integer".to_string()),
            },
        },
        _ => Err("to_int expects a string".to_string()),
    }
}

// Method only - String, Array, Hash
pub fn bisempty_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => Ok(Object::Boolean(s.is_empty())),
        Some(Object::Array(arr)) => Ok(Object::Boolean(arr.is_empty())),
        Some(Object::Hash(hash)) => Ok(Object::Boolean(hash.is_empty())),
        _ => Err("Invalid arguments to is_empty()".to_string()),
    }
}

// Method only - String, Array, Hash
pub fn blen_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => Ok(Object::Integer(s.len() as i64)),
        Some(Object::Array(arr)) => Ok(Object::Integer(arr.len() as i64)),
        Some(Object::Hash(hash)) => Ok(Object::Integer(hash.len() as i64)),
        _ => Err("len() requires a string or array".to_string()),
    }
}

// Method only - Hash, Array
pub fn bremove_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Hash(mut hash)), Some(key)) => match &key {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                hash.remove(&key);
                Ok(Object::Hash(hash))
            }
            _ => Err(format!("{} is not hashable", key.type_name())),
        },
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

// Method only - String, Array, Hash
pub fn bget_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::Integer(idx))) => {
            let i = idx;
            if i < 0 {
                return Err(format!("Index {} is negative", i));
            }
            let index = i as usize;
            let chars: Vec<char> = s.chars().collect();
            if index >= chars.len() {
                return Err(format!(
                    "Index {} out of bounds for string of length {}",
                    i,
                    chars.len()
                ));
            }
            Ok(Object::String(chars[index].to_string()))
        }
        (Some(Object::Array(vec)), Some(Object::Integer(idx))) => {
            let i = idx;
            if i < 0 {
                return Err(format!("Index {} is negative", i));
            }
            let index = i as usize;
            if index >= vec.len() {
                return Err(format!(
                    "Index {} out of bounds for array of length {}",
                    i,
                    vec.len()
                ));
            }
            Ok(vec[index].clone())
        }
        (Some(Object::Hash(hash)), Some(key)) => match &key {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                Ok(hash.get(&key).cloned().unwrap_or(Object::Null))
            }
            _ => Err(format!("{} is not hashable", key.type_name())),
        },
        _ => Err("get() requires (hash, key), (array, index), or (string, index)".to_string()),
    }
}
