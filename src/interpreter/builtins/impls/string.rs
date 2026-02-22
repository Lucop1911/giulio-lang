use crate::interpreter::obj::Object;

// Method only
pub fn btoupper_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => Ok(Object::String(s.to_uppercase())),
        Some(o) => Err(format!("to_upper() expects string, got {}", o.type_name())),
        None => Err(format!("to_upper() expects 1 argument, got 0")),
    }
}

// Method only
pub fn btolower_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => Ok(Object::String(s.to_lowercase())),
        Some(o) => Err(format!("to_lower() expects a string, got {}", o.type_name())),
        None => Err(format!("to_lower() expects 1 argument, got 0"))
    }
}

// Method only
pub fn bstartswith_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(prefix))) => {
            Ok(Object::Boolean(s.starts_with(&prefix)))
        }
        (Some(o), _) => Err(format!(
            "starts_with() expects string, got {}",
            o.type_name()
        )),
        (None, _) => Err(format!("starts_with() expects 2 arguments, got 1")),
    }
}

// Method only
pub fn bendswith_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(suffix))) => {
            Ok(Object::Boolean(s.ends_with(&suffix)))
        }
        (Some(o), _) => Err(format!("ends_with() expects string, got {}", o.type_name())),
        (None, _) => Err(format!("ends_with() expects 2 arguments, got 1")),
    }
}

pub fn breplace_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(old)), Some(Object::String(new))) => {
            let new_string = s.replace(&old, &new);
            Ok(Object::String(new_string))
        }
        (Some(o), _, _) => Err(format!("replace() expects string, got {}", o.type_name())),
        (None, Some(_), Some(_)) => Err(format!("replace() expects 3 arguments, got 1")),
        (_, None, Some(_)) => Err(format!("replace() expects 3 arguments, got 2")),
        (_, _, None) => Err(format!("replace() expects 3 arguments, got 3")),
    }
}

pub fn bsplit_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::String(s)), Some(Object::String(delimiter))) => {
            let parts: Vec<Object> = s
                .split(delimiter.as_str())
                .map(|part| Object::String(part.to_string()))
                .collect();
            Ok(Object::Array(parts))
        }
        (Some(o), _) => Err(format!("split() expects string, got {}", o.type_name())),
        (None, _) => Err(format!("split() expects 2 arguments, got 1")),
    }
}

pub fn btrim_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::String(s)) => {
            let trimmed_str = s.trim().to_string();
            Ok(Object::String(trimmed_str))
        }
        Some(o) => Err(format!("trim() expects string, got {}", o.type_name())),
        None => Err(format!("trim() expects 1 argument, got 0")),
    }
}
