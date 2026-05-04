use std::hash::BuildHasherDefault;
use ahash::{AHasher, HashMapExt};
use crate::vm::obj::Object;

type HashMap<K, V> = std::collections::HashMap<K, V, BuildHasherDefault<AHasher>>;

// Method only
pub(crate) fn bset_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next()) {
        (Some(Object::Hash(mut hash)), Some(key), Some(value)) => match &key {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                hash.insert(key, value);
                Ok(Object::Hash(hash))
            }
            _ => Err(format!(
                "set() key must be integer, boolean, or string, got {}",
                key.type_name()
            )),
        },
        (Some(o), _, _) => Err(format!("set() expects hash, got {}", o.type_name())),
        (None, _, _) => Err(format!("set() expects 3 arguments, got {}", args.len() + 1)),
    }
}

// Method only
pub(crate) fn bhas_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Hash(hash)), Some(key)) => match &key {
            Object::Integer(_) | Object::Boolean(_) | Object::String(_) => {
                Ok(Object::Boolean(hash.contains_key(&key)))
            }
            _ => Err(format!(
                "has() key must be integer, boolean, or string, got {}",
                key.type_name()
            )),
        },
        (Some(o), _) => Err(format!("has() expects hash, got {}", o.type_name())),
        (None, _) => Err("has() expects 2 arguments, got 1".to_string()),
    }
}

pub(crate) fn bkeys_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Hash(hash)) => {
            let keys: Vec<Object> = hash.keys().cloned().collect();
            Ok(Object::Array(Box::new(keys)))
        }
        Some(o) => Err(format!("keys() expects hash, got {}", o.type_name())),
        None => Err("keys() expects 1 argument, got 0".to_string()),
    }
}

pub(crate) fn bvalues_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Hash(hash)) => {
            let values: Vec<Object> = hash.values().cloned().collect();
            Ok(Object::Array(Box::new(values)))
        }
        Some(o) => Err(format!("values() expects hash, got {}", o.type_name())),
        None => Err("values() expects 1 argument, got 0".to_string()),
    }
}

pub(crate) fn bclear_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Hash(_)) => Ok(Object::Hash(Box::new(HashMap::new()))),
        Some(o) => Err(format!("clear() expects hash, got {}", o.type_name())),
        None => Err("clear() expects 1 argument, got 0".to_string()),
    }
}
