use std::collections::HashMap;
use crate::interpreter::obj::Object;

// Method only
pub fn bget_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Hash(hash)), Some(key)) => {
            match &key {
                Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                    Ok(hash.get(&key).cloned().unwrap_or(Object::Null))
                }
                _ => Err(format!("{} is not hashable", key.type_name())),
            }
        }
        _ => Err("Invalid arguments to get(hash, key)".to_string()),
    }
}

// Method only
pub fn bset_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next()) {
        (Some(Object::Hash(mut hash)), Some(key), Some(value)) => {
            match &key {
                Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                    hash.insert(key, value);
                    Ok(Object::Hash(hash))
                }
                _ => Err(format!("{} is not hashable", key.type_name())),
            }
        }
        _ => Err("Invalid arguments to set(hash, key, value)".to_string()),
    }
}

// Method only
pub fn bhas_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Hash(hash)), Some(key)) => {
            match &key {
                Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                    Ok(Object::Boolean(hash.contains_key(&key)))
                }
                _ => Err(format!("{} is not hashable", key.type_name())),
            }
        }
        _ => Err("Invalid arguments to has(hash, key)".to_string()),
    }
}

pub fn bkeys_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Hash(hash)) => {
            let keys: Vec<Object> = hash.keys().cloned().collect();
            Ok(Object::Array(keys))
        }
        _ => Err("Invalid arguments to keys(hash)".to_string()),
    }
}

pub fn bvalues_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Hash(hash)) => {
            let values: Vec<Object> = hash.values().cloned().collect();
            Ok(Object::Array(values))
        }
        _ => Err("Invalid arguments to values(hash)".to_string()),
    }
}

pub fn bclear_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Hash(_)) => {
            Ok(Object::Hash(HashMap::new()))
        }
        _ => Err("Invalid arguments to clear(hash)".to_string()),
    }
}