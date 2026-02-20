use crate::interpreter::obj::Object;

pub fn bhead_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Array(arr)) => match arr.into_iter().next() {
            None => Err(format!("head() cannot get head of empty array")),
            Some(x) => Ok(x),
        },
        Some(o) => Err(format!("head() expects array, got {}", o.type_name())),
        None => Err(format!("head() expects 1 argument, got 0")),
    }
}

pub fn btail_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.into_iter().next() {
        Some(Object::Array(mut arr)) => match arr.len() {
            0 => Err(format!("tail() cannot get tail of empty array")),
            _ => {
                arr.remove(0);
                Ok(Object::Array(arr))
            }
        },
        Some(o) => Err(format!("tail() expects array, got {}", o.type_name())),
        None => Err(format!("tail() expects 1 argument, got 0")),
    }
}

pub fn bcons_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(o), Some(Object::Array(mut os))) => {
            os.insert(0, o);
            Ok(Object::Array(os))
        }
        (Some(o), Some(other)) => Err(format!(
            "cons() expects (value, array), got {} and {}",
            o.type_name(),
            other.type_name()
        )),
        (None, _) => Err(format!("cons() expects 2 arguments, got 1")),
        (_, None) => Err(format!("cons() expects 2 arguments, got 1")),
    }
}

pub fn bpush_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Array(mut arr)), Some(o)) => {
            arr.push(o);
            Ok(Object::Array(arr))
        }
        (Some(o), _) => Err(format!("push() expects array, got {}", o.type_name())),
        (None, _) => Err(format!("push() expects 2 arguments, got 1")),
    }
}
