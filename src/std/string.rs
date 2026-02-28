use crate::errors::RuntimeError;
use crate::interpreter::obj::Object;

pub fn string_join(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match (&args[0], &args[1]) {
        (Object::Array(arr), Object::String(separator)) => {
            let strings: Result<Vec<String>, RuntimeError> = arr
                .iter()
                .map(|obj| match obj {
                    Object::String(s) => Ok(s.clone()),
                    o => Err(RuntimeError::TypeMismatch {
                        expected: "string".to_string(),
                        got: o.type_name(),
                    }),
                })
                .collect();

            match strings {
                Ok(strs) => Ok(Object::String(strs.join(separator))),
                Err(e) => Err(e),
            }
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "array, string".to_string(),
            got: "invalid arguments".to_string(),
        }),
    }
}

pub fn string_reverse(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match args.first() {
        Some(Object::String(s)) => {
            let mut chars: Vec<char> = s.chars().collect();
            chars.reverse();
            Ok(Object::String(chars.into_iter().collect()))
        }
        Some(o) => Err(RuntimeError::TypeMismatch {
            expected: "string".to_string(),
            got: o.type_name(),
        }),
        None => Err(RuntimeError::WrongNumberOfArguments {
            min: 1,
            max: 1,
            got: 0,
        }),
    }
}

pub fn string_repeat(args: Vec<Object>) -> Result<Object, RuntimeError> {
    match (&args[0], &args[1]) {
        (Object::String(s), Object::Integer(n)) => {
            if *n < 0 {
                return Err(RuntimeError::InvalidArguments(
                    "repeat count must be non-negative".to_string(),
                ));
            }
            Ok(Object::String(s.repeat(*n as usize)))
        }
        _ => Err(RuntimeError::TypeMismatch {
            expected: "string, integer".to_string(),
            got: "invalid arguments".to_string(),
        }),
    }
}
