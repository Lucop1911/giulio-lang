use crate::vm::obj::Object;

pub(crate) fn bpow_fn(args: Vec<Object>) -> Result<Object, String> {
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
        (Some(o), _) => Err(format!("pow() expects integer, got {}", o.type_name())),
        (None, _) => Err("pow() expects 2 arguments, got 1".to_string()),
    }
}

pub(crate) fn babs_fn(args: Vec<Object>) -> Result<Object, String> {
    match args.first() {
        Some(Object::Integer(x)) => Ok(Object::Integer(x.abs())),
        Some(o) => Err(format!("abs() expects integer, got {}", o.type_name())),
        None => Err("abs() expects 1 argument, got 0".to_string()),
    }
}

pub(crate) fn bmin_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args.first(), args.get(1)) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => Ok(Object::Integer((*a).min(*b))),
        (Some(o), _) => Err(format!("min() expects integer, got {}", o.type_name())),
        (None, _) => Err("min() expects 2 arguments, got 1".to_string()),
    }
}

pub(crate) fn bmax_fn(args: Vec<Object>) -> Result<Object, String> {
    match (args.first(), args.get(1)) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => Ok(Object::Integer((*a).max(*b))),
        (Some(o), _) => Err(format!("max() expects integer, got {}", o.type_name())),
        (None, _) => Err("max() expects 2 arguments, got 1".to_string()),
    }
}
