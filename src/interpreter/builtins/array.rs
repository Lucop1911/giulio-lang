use crate::interpreter::obj::Object;

pub fn blen_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::String(s)) => Ok(Object::Integer(s.len() as i64)),
        Some(Object::Array(arr)) => Ok(Object::Integer(arr.len() as i64)),
        _ => Err("len() requires a string or array".to_string()),
    }
}

pub fn bhead_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Array(arr)) => match arr.into_iter().next() {
            None => Err("cannot get head of empty array".to_string()),
            Some(x) => Ok(x),
        },
        _ => Err("head() requires an array".to_string()),
    }
}

pub fn btail_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Array(mut arr)) => match arr.len() {
            0 => Err("cannot get tail of empty array".to_string()),
            _ => {
                arr.remove(0);
                Ok(Object::Array(arr))
            }
        },
        _ => Err("tail() requires an array".to_string()),
    }
}

pub fn bcons_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(o), Some(Object::Array(mut os))) => {
            os.insert(0, o);
            Ok(Object::Array(os))
        }
        _ => Err("cons() requires a value and an array".to_string()),
    }
}

pub fn bpush_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Array(mut arr)), Some(o)) => {
            arr.push(o);
            Ok(Object::Array(arr))
        }
        _ => Err("push() requires an array and a value".to_string()),
    }
}