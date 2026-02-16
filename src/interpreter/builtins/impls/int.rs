use crate::interpreter::obj::Object;

pub fn bpow_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args.first(), args.get(1)) {
        (Some(Object::Integer(base)), Some(Object::Integer(exp))) => {
            if *exp < 0 {
                return Err("pow() does not support negative exponents".to_string());
            }
            
            match (*base).checked_pow(*exp as u32) {
                Some(result) => Ok(Object::Integer(result)),
                None => Err("pow() result overflow".to_string()),
            }
        }
        _ => Err("pow() expects two integers".to_string())
    }
}

pub fn babs_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Integer(x)) => {
            Ok(Object::Integer(x.abs()))
        }
        _ => {
            Err("abs() expects an integer".to_string())
        }
    }
}

pub fn bmin_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args.first(), args.get(1)) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => {
            Ok(Object::Integer((*a).min(*b)))
        }
        _ => {
            Err("min() expects two integers".to_string())
        }
    }
}

pub fn bmax_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args.first(), args.get(1)) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => {
            Ok(Object::Integer((*a).max(*b)))
        }
        _ => {
            Err("max() expects two integers".to_string())
        }
    }
}