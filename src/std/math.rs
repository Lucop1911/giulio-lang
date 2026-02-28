use crate::errors::RuntimeError;
use crate::interpreter::obj::Object;
use rand::Rng;

pub fn math_clamp(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Object::Integer(n), Object::Integer(min), Object::Integer(max)) => {
            if min > max {
                return Err(RuntimeError::InvalidArguments(
                    "min cannot be greater than max".to_string(),
                ));
            }

            Ok(Object::Integer(*n.clamp(min, max)))
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "integer, integer, integer".to_string(),
            got: "invalid arguments".to_string(),
        }),
    }
}

pub fn math_random(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut rng = rand::rng();

    match args.as_slice() {
        [] => Ok(Object::Integer(rng.random_range(0..=10))),
        [Object::Integer(max)] => {
            if *max < 0 {
                return Err(RuntimeError::InvalidArguments(
                    "max must be non negative".to_string(),
                ));
            }
            Ok(Object::Integer(rng.random_range(0..*max)))
        }
        [Object::Integer(min), Object::Integer(max)] => {
            if *max < *min {
                return Err(RuntimeError::InvalidArguments(
                    "min must be lower than or equal to max".to_string(),
                ));
            }
            Ok(Object::Integer(rng.random_range(*min..=*max)))
        }
        _ => Err(RuntimeError::InvalidArguments(
            "random() expects 0, 1, or 2 integer arguments".to_string(),
        )),
    }
}

pub fn math_round(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => Ok(Object::Float(n.round())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_floor(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => Ok(Object::Float(n.floor())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_ceil(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => Ok(Object::Float(n.ceil())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_sqrt(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => {
            if *n < 0.0 {
                return Err(RuntimeError::InvalidArguments(
                    "sqrt argument must be non-negative".to_string(),
                ));
            }
            Ok(Object::Float(n.sqrt()))
        }
        Some(Object::Integer(n)) => {
            if *n < 0 {
                return Err(RuntimeError::InvalidArguments(
                    "sqrt argument must be non-negative".to_string(),
                ));
            }
            Ok(Object::Float((*n as f64).sqrt()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float or integer".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_sin(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => Ok(Object::Float(n.sin())),
        Some(Object::Integer(n)) => Ok(Object::Float((*n as f64).sin())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float or integer".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_cos(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => Ok(Object::Float(n.cos())),
        Some(Object::Integer(n)) => Ok(Object::Float((*n as f64).cos())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float or integer".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_tan(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => Ok(Object::Float(n.tan())),
        Some(Object::Integer(n)) => Ok(Object::Float((*n as f64).tan())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float or integer".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_log(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => {
            if *n <= 0.0 {
                return Err(RuntimeError::InvalidArguments(
                    "log argument must be positive".to_string(),
                ));
            }
            Ok(Object::Float(n.ln()))
        }
        Some(Object::Integer(n)) => {
            if *n <= 0 {
                return Err(RuntimeError::InvalidArguments(
                    "log argument must be positive".to_string(),
                ));
            }
            Ok(Object::Float((*n as f64).ln()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float or integer".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_log10(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => {
            if *n <= 0.0 {
                return Err(RuntimeError::InvalidArguments(
                    "log10 argument must be positive".to_string(),
                ));
            }
            Ok(Object::Float(n.log10()))
        }
        Some(Object::Integer(n)) => {
            if *n <= 0 {
                return Err(RuntimeError::InvalidArguments(
                    "log10 argument must be positive".to_string(),
                ));
            }
            Ok(Object::Float((*n as f64).log10()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "float or integer".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_abs_int(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Integer(n)) => Ok(Object::Integer(n.abs())),
        Some(Object::Float(n)) => Ok(Object::Float(n.abs())),
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "integer or float".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn math_min_int(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => Ok(Object::Integer(a.min(b))),
        (Some(Object::Float(a)), Some(Object::Float(b))) => Ok(Object::Float(a.min(b))),
        (Some(Object::Integer(a)), Some(Object::Float(b))) => Ok(Object::Float((a as f64).min(b))),
        (Some(Object::Float(a)), Some(Object::Integer(b))) => Ok(Object::Float(a.min(b as f64))),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "integer or float, integer or float".to_string(),
            got: "invalid arguments".to_string(),
        }),
    }
}

pub fn math_max_int(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(Object::Integer(a)), Some(Object::Integer(b))) => Ok(Object::Integer(a.max(b))),
        (Some(Object::Float(a)), Some(Object::Float(b))) => Ok(Object::Float(a.max(b))),
        (Some(Object::Integer(a)), Some(Object::Float(b))) => Ok(Object::Float((a as f64).max(b))),
        (Some(Object::Float(a)), Some(Object::Integer(b))) => Ok(Object::Float(a.max(b as f64))),
        _ => Err(RuntimeError::TypeMismatch {
            expected: "integer or float, integer or float".to_string(),
            got: "invalid arguments".to_string(),
        }),
    }
}

pub fn math_pi() -> Object {
    Object::Float(std::f64::consts::PI)
}

pub fn math_e() -> Object {
    Object::Float(std::f64::consts::E)
}
