use crate::interpreter::obj::Object;

// Method only
pub fn btostring_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Integer(n)) => {
            Ok(Object::String(n.to_string()))
        }
        _ => {
            Err("to_string() expects an integer".to_string())
        }
    }
}

pub fn bpow_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Integer(n)), Some(Object::Integer(power))) => {
            Ok(Object::Integer(n.pow(power as u32)))
        }
        _ => Err("Invalid arguments to pow()".to_string())
    }
}

pub fn babs_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.get(0) {
        Some(Object::Integer(x)) => {
            Ok(Object::Integer(x.abs()))
        }
        _ => {
            Err("Invalid arguments to abs()".to_string())
        }
    }
}

pub fn bmin_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    
    match (args.next(), args.next()) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => {
            Ok(Object::Integer(a.min(b)))
        }
        _ => {
            Err("Invalid arguments to min()".to_string())
        }
    }
}

pub fn bmax_fn(args: Vec<Object>) -> Result<Object, String> {
    let mut args = args.into_iter();
    
    match (args.next(), args.next()) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => {
            Ok(Object::Integer(a.max(b)))
        }
        _ => {
            Err("Invalid arguments to max()".to_string())
        }
    }
}