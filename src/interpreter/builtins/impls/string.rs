use crate::interpreter::obj::Object;

// Method only
pub fn btoint_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(s)) => {
            match s.trim().parse::<i64>() {
                Ok(n) => Ok(Object::Integer(n)),
                Err(_) => Err("Cannot convert string to int".to_string()),
            }
        }
        _ => Err("to_int expects a string".to_string()),
    }
}

pub fn bisempty_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(s)) => {
            Ok(Object::Boolean(s.is_empty()))
        }
        Some(Object::Array(arr)) => {
            Ok(Object::Boolean(arr.is_empty()))
        }
        _ => Err("Invalid arguments to is_empty()".to_string())
    }
}

pub fn bstartswith_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(prefix))) => {
            Ok(Object::Boolean(s.starts_with(&prefix)))
        }
        _ => Err("Invalid arguments to starts_with(string, prefix)".to_string()),
    }
}

pub fn bendswith_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(prefix))) => {
            Ok(Object::Boolean(s.ends_with(&prefix)))
        }
        _ => Err("Invalid arguments to starts_with(string, prefix)".to_string()),
    }
}

pub fn breplace_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(old)), Some(Object::String(new))) => {
            let new_string = s.replace(&old, &new);
            Ok(Object::String(new_string))
        }
        _ => Err("Invalid arguments to replace(string, from, to)".to_string())
    }
}