use num_bigint::ToBigInt;
use num_traits::ToPrimitive;

use crate::interpreter::obj::Object;

// Method only
pub fn btostring_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(obj) => Ok(Object::String(format!("{}", obj))),
        _ => Err(format!(
            "to_string() expects 1 argument, got {}",
            args.len()
        )),
    }
}

// Method only
pub fn btoint_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => match s.trim().parse::<i64>() {
            Ok(n) => Ok(Object::Integer(n)),
            Err(_) => Err(format!("to_int() cannot convert '{}' to integer", s)),
        },
        Some(Object::Float(n)) => match n.to_i64() {
            Some(n) => Ok(Object::Integer(n)),
            None => match n.to_bigint() {
                Some(big) => Ok(Object::BigInteger(big)),
                None => Err(format!(
                    "to_int() cannot convert {} to integer (overflow)", n
                )),
            },
        },
        Some(o) => Err(format!(
            "to_int() expects string or float, got {}",
            o.type_name()
        )),
        None => Err(format!("to_int() expects 1 argument, got 0")),
    }
}

// Method only - String, Array, Hash
pub fn bisempty_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => Ok(Object::Boolean(s.is_empty())),
        Some(Object::Array(arr)) => Ok(Object::Boolean(arr.is_empty())),
        Some(Object::Hash(hash)) => Ok(Object::Boolean(hash.is_empty())),
        Some(o) => Err(format!(
            "is_empty() expects string, array, or hash, got {}",
            o.type_name()
        )),
        None => Err(format!("is_empty() expects 1 argument, got 0")),
    }
}

// Method only - String, Array, Hash
pub fn blen_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => Ok(Object::Integer(s.len() as i64)),
        Some(Object::Array(arr)) => Ok(Object::Integer(arr.len() as i64)),
        Some(Object::Hash(hash)) => Ok(Object::Integer(hash.len() as i64)),
        Some(o) => Err(format!(
            "len() expects string, array, or hash, got {}",
            o.type_name()
        )),
        None => Err(format!("len() expects 1 argument, got 0")),
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
            _ => Err(format!(
                "remove() key must be integer, boolean, or string, got {}",
                key.type_name()
            )),
        },
        (Some(Object::Array(mut vec)), Some(Object::Integer(idx))) => {
            let i = idx as isize;
            if i < 0 || i as usize >= vec.len() {
                return Err(format!(
                    "remove() index {} out of bounds (array length: {})",
                    i,
                    vec.len()
                ));
            }
            let _ = vec.remove(i as usize);
            Ok(Object::Array(vec))
        }
        (Some(o), _) => Err(format!(
            "remove() expects hash or array, got {}",
            o.type_name()
        )),
        (None, _) => Err(format!("remove() expects 2 arguments, got 1")),
    }
}

// Method only - String, Array, Hash
pub fn bget_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::Integer(idx))) => {
            let i = idx;
            if i < 0 {
                return Err(format!("get() index {} is negative", i));
            }
            let index = i as usize;
            let chars: Vec<char> = s.chars().collect();
            if index >= chars.len() {
                return Err(format!(
                    "get() index {} out of bounds (string length: {})",
                    i,
                    chars.len()
                ));
            }
            Ok(Object::String(chars[index].to_string()))
        }
        (Some(Object::Array(vec)), Some(Object::Integer(idx))) => {
            let i = idx;
            if i < 0 {
                return Err(format!("get() index {} is negative", i));
            }
            let index = i as usize;
            if index >= vec.len() {
                return Err(format!(
                    "get() index {} out of bounds (array length: {})",
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
            _ => Err(format!(
                "get() key must be integer, boolean, or string, got {}",
                key.type_name()
            )),
        },
        (Some(o), _) => Err(format!(
            "get() expects hash, array, or string, got {}",
            o.type_name()
        )),
        (None, _) => Err(format!("get() expects 2 arguments, got {}", args.len() + 1)),
    }
}
