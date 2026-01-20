use crate::interpreter::obj::Object;

// Method only
pub fn btoint_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => {
            match s.trim().parse::<i64>() {
                Ok(n) => Ok(Object::Integer(n)),
                Err(_) => Err("Cannot convert string to int".to_string()),
            }
        }
        _ => Err("to_int expects a string".to_string()),
    }
}

// Method only
pub fn bstartswith_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(prefix))) => {
            Ok(Object::Boolean(s.starts_with(&prefix)))
        }
        _ => Err("Invalid arguments to starts_with(string, prefix)".to_string()),
    }
}

// Method only
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

pub fn bsplit_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(delimiter))) => {
            let parts: Vec<Object> = s.split(delimiter.as_str())
                .map(|part| Object::String(part.to_string()))
                .collect();
            Ok(Object::Array(parts))
        }
        _ => Err("split expects two strings".to_string()),
    }
}

pub fn btrim_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => {
            let trimmed_str = s.trim().to_string();
            Ok(Object::String(trimmed_str))
        }
        _ => Err("Invalid arguments for trim(). trim() expects 1 string".to_string())
    }
}