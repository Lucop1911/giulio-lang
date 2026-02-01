use crate::interpreter::obj::Object;
use crate::errors::RuntimeError;
use rand::Rng;

pub fn math_clamp(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match (&args[0], &args[1], &args[2]) {
        (Object::Integer(n), Object::Integer(min), Object::Integer(max)) => {
            if min > max { 
                return Err(RuntimeError::InvalidArguments("min cannot be greater than max".to_string()))
            }

            Ok(Object::Integer(*n.clamp(min, max)))
        }
        _ => Err(RuntimeError::TypeMismatch { expected: "integer, integer, integer".to_string(), got: "invalid arguments".to_string() })
    }
}

pub fn math_random(args: Vec<Object>) -> Result<Object, RuntimeError> {
    let mut rng = rand::rng();

    match args.as_slice() {
        [] => Ok(Object::Integer(rng.random_range(0..=10))),
        [Object::Integer(max)] => {
            if *max < 0 { 
                return Err(RuntimeError::InvalidArguments("max must be non negative".to_string()))
            }
            Ok(Object::Integer(rng.random_range(0..*max)))
        }
        [Object::Integer(min), Object::Integer(max)] => {
            if *max < *min {
                return Err(RuntimeError::InvalidArguments("min must be lower than or equal to max".to_string()))
            }
            Ok(Object::Integer(rng.random_range(*min..=*max)))
        }
        _ => Err(RuntimeError::InvalidArguments("random() expects 0, 1, or 2 integer arguments".to_string())),
    }
}

pub fn math_round(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::Float(n)) => {
            Ok(Object::Float(n.round()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch { expected: "float".to_string(), got: o.type_name() }),
        None => Err(RuntimeError::WrongNumberOfArguments { min: 1, max: 1, got: 0 }),
    }
}